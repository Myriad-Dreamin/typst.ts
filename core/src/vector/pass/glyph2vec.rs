//! Lowering Typst Document into SvgItem.

use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Arc;

use parking_lot::Mutex;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use ttf_parser::GlyphId;
use typst::layout::Size;
use typst::text::Font;
use typst::visualize::Image;

use crate::font::GlyphProvider;

use crate::vector::ir::{self, FlatGlyphItem, FontItem, FontPack, FontRef, GlyphItem, GlyphRef};

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
    font_mapping: crate::adt::CHashMap<Font, FontRef>,
    /// Detect font short hash conflict
    font_conflict_checker: crate::adt::CHashMap<u32, Font>,
    /// Lock to get a unique local index for each font.
    font_write_lock: Mutex<()>,

    /// Intermediate representation of an incompleted glyph pack.
    glyph_defs: crate::adt::CHashMap<GlyphItem, (GlyphRef, FontRef)>,

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
            font_write_lock: Default::default(),
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
        let fonts = fonts.into_iter().map(|(a, _)| a.into()).collect();

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

    pub fn build_font(&self, font: &Font) -> FontRef {
        if let Some(id) = self.font_mapping.get(font) {
            return *id;
        }
        let _write_lock = self.font_write_lock.lock();

        let new_abs_ref = RefCell::new(FontRef {
            hash: 0xfffe,
            idx: 0xfffe,
        });

        self.font_mapping.alter(font.clone(), |e| {
            if e.is_some() {
                *new_abs_ref.borrow_mut() = *e.as_ref().unwrap();
                return e;
            }

            let mut abs_ref = FontRef {
                hash: fxhash::hash32(font),
                idx: self.font_mapping.len() as u32,
            };

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

            *new_abs_ref.borrow_mut() = abs_ref;
            if ENABLE_REF_CNT {
                self.new_fonts.lock().push(font.clone().into());
            }

            Some(abs_ref)
        });

        new_abs_ref.into_inner()
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
    fn raw_glyph(&self, _font: &Font, _id: GlyphId) -> Option<GlyphItem> {
        None
    }
}

#[cfg(feature = "glyph2vec")]
impl ConvertInnerImpl {
    fn ligature_len(&self, font: &Font, id: GlyphId) -> u8 {
        if !self.lowering_ligature {
            return 0;
        }

        self.gp
            .ligature_glyph(font, id)
            .map(|l| l.len())
            .unwrap_or_default() as u8
    }

    fn raw_glyph(&self, font: &Font, id: GlyphId) -> Option<GlyphItem> {
        self.svg_glyph(font, id)
            .map(GlyphItem::Image)
            .or_else(|| self.bitmap_glyph(font, id).map(GlyphItem::Image))
            .or_else(|| self.outline_glyph(font, id).map(GlyphItem::Outline))
    }

    /// Lower an SVG glyph into svg item.
    /// More information: https://learn.microsoft.com/zh-cn/typography/opentype/spec/svg
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ir::ImageGlyphItem>> {
        use crate::vector::ir::Scalar;
        use crate::vector::utils::AbsExt;

        let image = Self::extract_svg_glyph(&self.gp, font, id)?;

        // position our image
        let ascender = font
            .metrics()
            .ascender
            .at(typst::layout::Abs::raw(font.metrics().units_per_em))
            .to_f32();

        Some(Arc::new(ir::ImageGlyphItem {
            ts: ir::Transform {
                sx: Scalar(1.),
                ky: Scalar(0.),
                kx: Scalar(0.),
                sy: Scalar(-1.),
                tx: Scalar(0.),
                ty: Scalar(ascender),
            },
            image,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower a bitmap glyph into the svg text.
    fn bitmap_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ir::ImageGlyphItem>> {
        use crate::vector::utils::AbsExt;
        /// Use types from `tiny-skia` crate.
        use tiny_skia as sk;

        let ppem = u16::MAX;
        let upem = font.metrics().units_per_em as f32;

        let (glyph_image, raster_x, raster_y) = self.gp.bitmap_glyph(font, id, ppem)?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?

        let w = glyph_image.width() as f64;
        let h = glyph_image.height() as f64;
        let sz = Size::new(typst::layout::Abs::raw(w), typst::layout::Abs::raw(h));

        let image = ir::ImageItem {
            image: Arc::new(glyph_image.into()),
            size: sz.into(),
        };

        // position our image
        // first, the ascender is used
        // next, also apply an offset of (1 - ascender) like typst
        let adjusted = font.metrics().ascender * 2. - typst::layout::Em::one();
        // let adjusted = font.metrics().ascender;

        let adjusted = adjusted
            .at(typst::layout::Abs::raw(font.metrics().units_per_em))
            .to_f32();

        let ts = sk::Transform::from_scale(upem / w as f32, -upem / h as f32);

        // size
        let dx = raster_x as f32;
        let dy = raster_y as f32;
        let ts = ts.post_translate(dx, adjusted + dy);

        Some(Arc::new(ir::ImageGlyphItem {
            ts: ts.into(),
            image,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower an outline glyph into svg text. This is the "normal" case.
    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ir::OutlineGlyphItem>> {
        let d = self.gp.outline_glyph(font, id)?.into();

        Some(Arc::new(ir::OutlineGlyphItem {
            ts: None,
            d,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    fn extract_svg_glyph(g: &GlyphProvider, font: &Font, id: GlyphId) -> Option<ir::ImageItem> {
        struct FindViewBoxResult<'a> {
            start_span: Option<xmlparser::StrSpan<'a>>,
            first_viewbox: Option<(xmlparser::StrSpan<'a>, xmlparser::StrSpan<'a>)>,
        }

        /// Find the string location of the **first** viewBox attribute.
        /// When there are multiple viewBox attributes, the first one is used
        /// (as many xml-based dom engines do).
        fn find_viewbox_attr(svg_str: &'_ str) -> FindViewBoxResult<'_> {
            let document = xmlparser::Tokenizer::from(svg_str);

            let mut start_span = None;
            let mut first_viewbox = None;
            for n in document {
                let tok = n.unwrap();
                match tok {
                    xmlparser::Token::ElementStart { span, local, .. } => {
                        if local.as_str() == "svg" {
                            start_span = Some(span);
                        }
                    }
                    xmlparser::Token::Attribute {
                        span, local, value, ..
                    } => {
                        if local.as_str() == "viewBox" {
                            first_viewbox = Some((span, value));
                            break;
                        }
                    }
                    xmlparser::Token::ElementEnd { .. } => break,
                    _ => {}
                }
            }

            FindViewBoxResult {
                start_span,
                first_viewbox,
            }
        }
        use crate::vector::utils::AbsExt;
        use std::io::Read;

        use once_cell::sync::OnceCell;

        static WARN_VIEW_BOX: OnceCell<()> = OnceCell::new();

        let data = g.svg_glyph(font, id)?;
        let mut data = data.as_ref();

        let font_metrics = font.metrics();

        // Decompress SVGZ.
        let mut decoded = vec![];

        // The first three bytes of the gzip-encoded document header must be
        //   0x1F, 0x8B, 0x08.
        if data.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = flate2::read::GzDecoder::new(data);
            decoder.read_to_end(&mut decoded).ok()?;
            data = &decoded;
        }

        // todo: It is also legal to provide a SVG document containing multiple glyphs.
        // > When a font engine renders glyph 14, the result shall be the same as
        // > rendering the following SVG document:
        // > `  <svg> <defs> <use #glyph{id}> </svg>`
        // See: <https://learn.microsoft.com/en-us/typography/opentype/spec/svg#glyph-identifiers>

        let upem = typst::layout::Abs::raw(font.units_per_em());
        let (width, height) = (upem.to_f32(), upem.to_f32());
        let origin_ascender = font_metrics.ascender.at(upem).to_f32();

        let doc_string = String::from_utf8(data.to_owned()).unwrap();

        // todo: verify SVG capability requirements and restrictions

        // Partially parse the view box attribute
        let mut svg_str = std::str::from_utf8(data).ok()?.to_owned();
        let FindViewBoxResult {
            start_span,
            first_viewbox,
        } = find_viewbox_attr(svg_str.as_str());

        // determine view box
        let view_box = first_viewbox
            .as_ref()
            .map(|s| {
                WARN_VIEW_BOX.get_or_init(|| {
                    println!(
                        "render_svg_glyph with viewBox, This should be helpful if you can help us verify the result: {:?} {:?}",
                        font.info().family,
                        doc_string
                    );
                });
                s.1.as_str().to_owned()
            })
            .unwrap_or_else(|| format!("0 {} {width} {height}", -origin_ascender));

        // determine view box
        match first_viewbox {
            Some((span, ..)) => {
                // replace the first viewBox attribute
                svg_str.replace_range(span.range(), format!(r#"viewBox="{view_box}""#).as_str());
            }
            None => {
                // insert viewBox attribute to the begin of svg tag
                svg_str.insert_str(
                    start_span.unwrap().range().end,
                    format!(r#" viewBox="{view_box}""#).as_str(),
                );
            }
        }

        let glyph_image = typst::visualize::Image::new(
            svg_str.as_bytes().to_vec().into(),
            typst::visualize::ImageFormat::Vector(typst::visualize::VectorFormat::Svg),
            // typst::geom::Axes::new(width as u32, height as u32),
            None,
        )
        .ok()?;

        let sz = Size::new(
            typst::layout::Abs::raw(glyph_image.width() as f64),
            typst::layout::Abs::raw(glyph_image.height() as f64),
        );

        Some(ir::ImageItem {
            image: Arc::new(glyph_image.into()),
            size: sz.into(),
        })
    }
}

/// Lower a raster or SVG image into svg item.
#[comemo::memoize]
fn lower_image(image: &Image, size: Size) -> ir::ImageItem {
    ir::ImageItem {
        image: Arc::new(image.clone().into()),
        size: size.into(),
    }
}
