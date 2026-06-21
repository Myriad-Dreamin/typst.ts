//! Lowering Typst Document into SvgItem.

use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Arc;

use parking_lot::Mutex;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use ttf_parser::GlyphId;
use typst::layout::Size;
use typst::text::color::{glyph_frame, should_outline, GlyphFrameItem};
use typst::text::FontInstance;
use typst::visualize::Image;

use crate::font::GlyphProvider;
use crate::ir::{self, FlatGlyphItem, FontItem, FontPack, FontRef, GlyphItem, GlyphRef};
use crate::IntoTypst;

pub type Glyph2VecPass = TGlyph2VecPass</* ENABLE_REF_CNT */ false>;
pub type IncrGlyph2VecPass = TGlyph2VecPass</* ENABLE_REF_CNT */ true>;

pub struct ConvertInnerImpl {
    /// A glyph backend provider.
    pub gp: GlyphProvider,

    /// Whether to lower ligature information
    pub lowering_ligature: bool,
}

/// Lower a glyph into vector item.
pub struct TGlyph2VecPass<const ENABLE_REF_CNT: bool = false> {
    pub inner: ConvertInnerImpl,

    /// Incremental state
    /// The lifetime of items, used to determine the lifetime of the new items.
    pub lifetime: u64,
    /// The new font items produced in this lifecycle.
    pub new_fonts: Mutex<Vec<FontItem>>,
    /// The new glyph items produced in this lifecycle.
    pub new_glyphs: Mutex<Vec<(GlyphRef, GlyphItem)>>,

    /// Intermediate representation of an incompleted font pack.
    /// All font items are stored in this map, and then sorted by the index.
    font_mapping: reflexo::adt::CHashMap<FontInstance, FontRef>,
    /// Detect font short hash conflict
    font_conflict_checker: reflexo::adt::CHashMap<u32, FontInstance>,
    /// Lock to get a unique local index for each font.
    font_index: Mutex<usize>,

    /// Intermediate representation of an incompleted glyph pack.
    glyph_defs: reflexo::adt::CHashMap<GlyphItem, (GlyphRef, FontRef)>,

    /// for interning
    pub used_fonts: HashSet<FontRef>,
    pub used_glyphs: HashSet<GlyphRef>,
}

impl<const ENABLE_REF_CNT: bool> TGlyph2VecPass<ENABLE_REF_CNT> {
    pub fn new(gp: GlyphProvider, lowering_ligature: bool) -> Self {
        Self {
            inner: ConvertInnerImpl::new(gp, lowering_ligature),

            lifetime: 0,
            font_mapping: Default::default(),
            font_conflict_checker: Default::default(),
            font_index: Default::default(),
            glyph_defs: Default::default(),
            new_fonts: Default::default(),
            new_glyphs: Default::default(),
            used_fonts: Default::default(),
            used_glyphs: Default::default(),
        }
    }

    pub fn finalize(&self) -> (FontPack, Vec<(GlyphRef, FlatGlyphItem)>) {
        let mut fonts = self.font_mapping.clone().into_iter().collect::<Vec<_>>();
        fonts.sort_by(|(_, a), (_, b)| a.idx.cmp(&b.idx));
        let fonts = fonts.into_iter().map(|(a, _)| a.into_typst()).collect();

        let glyphs = self.glyph_defs.clone().into_iter().collect::<Vec<_>>();
        let glyphs = glyphs
            .into_par_iter()
            .flat_map(|(a, b)| {
                self.inner.must_flat_glyph(&a).map(|g| {
                    (
                        GlyphRef {
                            font_hash: b.1.hash,
                            glyph_idx: b.0.glyph_idx,
                        },
                        g,
                    )
                })
            })
            .collect();

        (fonts, glyphs)
    }

    pub fn build_font(&self, font: &FontInstance) -> FontRef {
        if let Some(id) = self.font_mapping.get(font) {
            return *id;
        }

        // Lock before insertion checking to ensure atomicity
        let mut font_index_lock = self.font_index.lock();

        let entry = self.font_mapping.entry(font.clone());
        let entry = entry.or_insert_with(|| {
            let font_index = font_index_lock.deref_mut();
            let mut abs_ref = FontRef {
                hash: reflexo::hash::hash32(font),
                idx: (*font_index) as u32,
            };
            *font_index += 1;

            // Detect font short hash conflict
            'conflict_detection: loop {
                if let Some(conflict) = self.font_conflict_checker.get(&abs_ref.hash) {
                    if *conflict != *font {
                        log::error!(
                            "font conflict detected: {} {:?} {:?}",
                            abs_ref.hash,
                            font,
                            conflict
                        );
                    }
                    abs_ref.hash += 1;
                    continue 'conflict_detection;
                }

                self.font_conflict_checker
                    .insert(abs_ref.hash, font.clone());
                break 'conflict_detection;
            }

            if ENABLE_REF_CNT {
                self.new_fonts.lock().push(font.clone().into_typst());
            }

            abs_ref
        });

        *entry.value()
    }

    pub fn build_glyph(&self, font_ref: FontRef, glyph: GlyphItem) -> GlyphRef {
        let (_, id) = match &glyph {
            GlyphItem::Raw(g, id) => (g, id),
            _ => todo!(),
        };

        let glyph_idx = id.0 as u32;

        let abs_ref = GlyphRef {
            font_hash: font_ref.hash,
            glyph_idx,
        };

        if self
            .glyph_defs
            .insert(glyph.clone(), (abs_ref, font_ref))
            .is_some()
        {
            return abs_ref;
        }

        if ENABLE_REF_CNT {
            self.new_glyphs.lock().push((abs_ref, glyph));
        }

        abs_ref
    }

    #[allow(dead_code)]
    pub(crate) fn verify_glyph(&self, id: GlyphRef, data: &GlyphItem) {
        if let Some(glyph) = self.glyph_defs.get(data) {
            assert_eq!(glyph.0, id);
        } else {
            panic!("glyph not found");
        }
    }
}

impl IncrGlyph2VecPass {
    pub fn finalize_delta(&self) -> (FontPack, Vec<(GlyphRef, FlatGlyphItem)>) {
        let fonts = std::mem::take(self.new_fonts.lock().deref_mut());
        let glyphs = std::mem::take(self.new_glyphs.lock().deref_mut());
        let glyphs = glyphs
            .into_par_iter()
            .flat_map(|(id, glyph)| {
                let glyph = self.inner.must_flat_glyph(&glyph);
                glyph.map(|glyph| (id, glyph))
            })
            .collect::<Vec<_>>();
        (fonts, glyphs)
    }
}

impl ConvertInnerImpl {
    pub fn new(gp: GlyphProvider, lowering_ligature: bool) -> Self {
        Self {
            gp,
            lowering_ligature: cfg!(feature = "experimental-ligature") && lowering_ligature,
        }
    }

    pub fn glyph(&self, glyph_item: &GlyphItem) -> Option<GlyphItem> {
        match glyph_item {
            GlyphItem::Raw(font, id) => self.raw_glyph(font, *id),
            GlyphItem::Image(..) | GlyphItem::Outline(..) => Some(glyph_item.clone()),
            GlyphItem::None => Some(GlyphItem::None),
        }
    }

    pub fn must_flat_glyph(&self, glyph_item: &GlyphItem) -> Option<FlatGlyphItem> {
        let glyph_item = self.glyph(glyph_item)?;
        match glyph_item {
            GlyphItem::Outline(i) => Some(FlatGlyphItem::Outline(i)),
            GlyphItem::Image(i) => Some(FlatGlyphItem::Image(i)),
            GlyphItem::None | GlyphItem::Raw(..) => None,
        }
    }

    #[cfg(not(feature = "glyph2vec"))]
    fn raw_glyph(&self, _font: &FontInstance, _id: GlyphId) -> Option<GlyphItem> {
        None
    }
}

#[cfg(feature = "glyph2vec")]
impl ConvertInnerImpl {
    fn ligature_len(&self, font: &FontInstance, id: GlyphId) -> u8 {
        if !self.lowering_ligature {
            return 0;
        }

        self.gp
            .ligature_glyph(font, id)
            .map(|l| l.len())
            .unwrap_or_default() as u8
    }

    fn raw_glyph(&self, font: &FontInstance, id: GlyphId) -> Option<GlyphItem> {
        if should_outline(font, id) {
            self.outline_glyph(font, id).map(GlyphItem::Outline)
        } else {
            self.frame_glyph(font, id).map(GlyphItem::Image)
        }
    }

    /// Lower a color glyph through Typst's v0.15 color-font frame logic.
    fn frame_glyph(&self, font: &FontInstance, id: GlyphId) -> Option<Arc<ir::ImageGlyphItem>> {
        let frame = glyph_frame(font, id.0)?;
        let GlyphFrameItem::Image(pos, image, size) = frame.item else {
            return None;
        };

        Some(Arc::new(ir::ImageGlyphItem {
            ts: ir::Transform {
                sx: ir::Scalar(1.),
                ky: ir::Scalar(0.),
                kx: ir::Scalar(0.),
                sy: ir::Scalar(-1.),
                tx: pos.x.into_typst(),
                ty: (-pos.y).into_typst(),
            },
            image: lower_image(&image, size),
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower an outline glyph into svg text. This is the "normal" case.
    fn outline_glyph(&self, font: &FontInstance, id: GlyphId) -> Option<Arc<ir::OutlineGlyphItem>> {
        let d = self.gp.outline_glyph(font, id)?.into();

        Some(Arc::new(ir::OutlineGlyphItem {
            ts: None,
            d,
            ligature_len: self.ligature_len(font, id),
        }))
    }
}

/// Lower a raster or SVG image into svg item.
#[comemo::memoize]
fn lower_image(image: &Image, size: Size) -> ir::ImageItem {
    ir::ImageItem {
        image: Arc::new(image.clone().into_typst()),
        size: size.into_typst(),
    }
}
