//! Intermediate representation of vector items.
//!
//! VectorDoc and Module Relation:
//!
//! ┌──────────────┐ serialize  ┌────────────────────────────────────┐
//! │[`FlatModule`]├───────────►│[`super::stream::BytesModuleStream`]│
//! └──────────────┘            └───────────┬────────────────────────┘
//!      ▲                                  │
//!      │flatten                           │implement
//!      │                                  ▼
//! ┌────┴─────┐        merge       ┌────────────────┐
//! │[`Module`]│◄───────────────────┤[`ModuleStream`]│
//! └────┬─────┘                    └───────┬────────┘
//!      │                                  │
//!      │Store data of                     │merge
//!      ▼                                  ▼
//! ┌───────────────┐  select layout ┌────────────────────┐
//! │[`VecDocument`]│◄───────────────┤[`MultiVecDocument`]│
//! └───────────────┘                └────────────────────┘

use core::fmt;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::Arc;

mod color;
mod compose;
pub mod geom;
pub mod layout;
pub mod module;
mod preludes;
mod primitives;
mod text;
mod visualize;

pub use color::*;
pub use compose::*;
pub use geom::*;
pub use layout::*;
pub use module::*;
pub use primitives::*;
pub use text::*;
pub use visualize::*;

use parking_lot::Mutex;
#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

use typst::text::Font;

use crate::font::FontGlyphProvider;
use crate::{font::GlyphProvider, hash::Fingerprint, TakeAs};

pub use crate::ImmutStr;

use super::Glyph2VecPass;

/// A vector item that is specialized for representing
/// [`typst::model::Document`] or its subtypes.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum VecItem {
    None,
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(TextItem),
    Item(TransformedRef),
    Group(GroupRef, Option<Size>),
    Gradient(Arc<GradientItem>),
    Pattern(Arc<PatternItem>),
    ContentHint(char),
}

/// Module with page references of a [`typst::model::Document`].
pub struct VecDocument {
    /// module containing all of the data related to this document.
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub pages: Vec<Page>,
}

/// Module with multiple documents in a module [`typst::model::Document`].
pub struct MultiVecDocument {
    /// module containing all of the data related to this document.
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub layouts: Vec<LayoutRegion>,
}

impl Default for MultiVecDocument {
    fn default() -> Self {
        let pages = LayoutRegionNode::new_pages(Default::default());
        Self {
            module: Default::default(),
            layouts: vec![LayoutRegion::new_single(pages)],
        }
    }
}

impl MultiVecDocument {
    #[cfg(feature = "rkyv")]
    pub fn from_slice(v: &[u8]) -> Self {
        type DocStream<'a> = super::stream::BytesModuleStream<'a>;

        let mut res = Self::default();
        res.merge_delta(&DocStream::from_slice(v).checkout_owned());
        res
    }

    pub fn merge_delta(&mut self, v: impl ModuleStream) {
        self.layouts = v.layouts().take();
        self.module.merge_delta(v);
    }
}

/// Describing reference to a page
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Page {
    /// Unique hash to content
    pub content: Fingerprint,
    /// Page size for cropping content
    pub size: Size,
}

impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Page({}, {:.3}x{:.3})",
            self.content.as_svg_id(""),
            self.size.x.0,
            self.size.y.0
        )
    }
}

/// Flatten transform item.
/// Item representing an `<g/>` element applied with a [`TransformItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct TransformedRef(pub TransformItem, pub Fingerprint);

/// Flatten group item.
/// Absolute positioning items at their corresponding points.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct GroupRef(pub Arc<[(Point, Fingerprint)]>);

/// Item representing an `<a/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LinkItem {
    /// The target of the link item.
    pub href: ImmutStr,
    /// The box size of the link item.
    pub size: Size,
}

/// Item representing all the transform that is applicable to a [`SvgItem`].
/// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/transform>
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum TransformItem {
    /// `matrix` transform.
    Matrix(Arc<Transform>),
    /// `translate` transform.
    Translate(Arc<Axes<Abs>>),
    /// `scale` transform.
    Scale(Arc<(Ratio, Ratio)>),
    /// `rotate` transform.
    Rotate(Arc<Scalar>),
    /// `skewX skewY` transform.
    Skew(Arc<(Ratio, Ratio)>),

    /// clip path.
    Clip(Arc<PathItem>),
}

/// See [`TransformItem`].
impl From<TransformItem> for Transform {
    fn from(value: TransformItem) -> Self {
        match value {
            TransformItem::Matrix(m) => *m,
            TransformItem::Scale(m) => Transform::from_scale(m.0, m.1),
            TransformItem::Translate(m) => Transform::from_translate(m.x, m.y),
            TransformItem::Rotate(_m) => todo!(),
            TransformItem::Skew(m) => Transform::from_skew(m.0, m.1),
            TransformItem::Clip(_m) => Transform::identity(),
        }
    }
}

/// Global style namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StyleNs {
    /// style that contains a single css rule: `fill: #color`.
    Fill,
}

/// A finished pack that stores all the font items.
pub type FontPack = Vec<FontItem>;

/// A finished pack that stores all the glyph items.
pub type GlyphPack = Vec<(DefId, FlatGlyphItem)>;

pub struct GlyphPackBuilderImpl<const ENABLE_REF_CNT: bool = false> {
    /// Intermediate representation of an incompleted font pack.
    font_mapping: chashmap::CHashMap<Font, FontRef>,
    font_write_lock: Mutex<()>,

    /// Intermediate representation of an incompleted glyph pack.
    glyph_defs: chashmap::CHashMap<GlyphItem, (GlyphRef, FontRef)>,
    glyph_write_lock: Mutex<()>,

    pub lifetime: u64,
    pub new_fonts: Mutex<Vec<FontItem>>,
    pub new_glyphs: Mutex<Vec<(DefId, GlyphItem)>>,

    /// for interning
    pub used_fonts: HashSet<FontRef>,
    pub used_glyphs: HashSet<GlyphRef>,
}

pub type GlyphPackBuilder = GlyphPackBuilderImpl</* ENABLE_REF_CNT */ false>;
pub type IncrGlyphPackBuilder = GlyphPackBuilderImpl</* ENABLE_REF_CNT */ true>;

impl<const ENABLE_REF_CNT: bool> Default for GlyphPackBuilderImpl<ENABLE_REF_CNT> {
    fn default() -> Self {
        Self {
            lifetime: 0,
            font_mapping: Default::default(),
            font_write_lock: Default::default(),
            glyph_defs: Default::default(),
            glyph_write_lock: Default::default(),
            new_fonts: Default::default(),
            new_glyphs: Default::default(),
            used_fonts: Default::default(),
            used_glyphs: Default::default(),
        }
    }
}

impl<const ENABLE_REF_CNT: bool> GlyphPackBuilderImpl<ENABLE_REF_CNT> {
    pub fn finalize(&self) -> (FontPack, Vec<(DefId, GlyphItem)>) {
        let mut fonts = self.font_mapping.clone().into_iter().collect::<Vec<_>>();
        fonts.sort_by(|(_, a), (_, b)| a.idx.cmp(&b.idx));
        let fonts = fonts.into_iter().map(|(a, _)| a.into()).collect();

        let mut glyphs = self.glyph_defs.clone().into_iter().collect::<Vec<_>>();
        glyphs.sort_by(|(_, a), (_, b)| a.0.glyph_idx.cmp(&b.0.glyph_idx));
        let glyphs = glyphs
            .into_iter()
            .map(|(a, b)| (DefId(b.1.idx as u64), a))
            .collect();

        (fonts, glyphs)
    }

    pub fn build_font(&self, font: &Font) -> FontRef {
        if let Some(id) = self.font_mapping.get(font) {
            return id.clone();
        }
        let _write_lock = self.font_write_lock.lock();

        let new_abs_ref = RefCell::new(FontRef {
            hash: 0xfffe,
            idx: 0xfffe,
        });

        self.font_mapping.alter(font.clone(), |e| {
            if e.is_some() {
                *new_abs_ref.borrow_mut() = e.as_ref().unwrap().clone();
                return e;
            }

            let abs_ref = FontRef {
                hash: fxhash::hash32(font),
                idx: self.font_mapping.len() as u32,
            };
            *new_abs_ref.borrow_mut() = abs_ref.clone();
            if ENABLE_REF_CNT {
                self.new_fonts.lock().push(font.clone().into());
            }
            Some(abs_ref)
        });

        new_abs_ref.into_inner()
    }

    pub fn build_glyph(&self, glyph: &GlyphItem) -> GlyphRef {
        if let Some(id) = self.glyph_defs.get(glyph) {
            return id.0.clone();
        }
        let _write_lock = self.glyph_write_lock.lock();

        let new_abs_ref = RefCell::new(GlyphRef {
            font_hash: 0xfffe,
            glyph_idx: 0xfffe,
        });

        self.glyph_defs.alter(glyph.clone(), |e| {
            if e.is_some() {
                *new_abs_ref.borrow_mut() = e.as_ref().unwrap().0.clone();
                return e;
            }

            let g = match glyph {
                GlyphItem::Raw(g, _) => g,
                _ => todo!(),
            };

            let font_ref = self.build_font(g);

            let glyph_idx = self.glyph_defs.len() as u32;

            let abs_ref = GlyphRef {
                font_hash: font_ref.hash,
                glyph_idx,
            };
            *new_abs_ref.borrow_mut() = abs_ref.clone();
            if ENABLE_REF_CNT {
                self.new_glyphs
                    .lock()
                    .push((DefId(font_ref.idx as u64), glyph.clone()));
            }
            Some((abs_ref, font_ref))
        });

        new_abs_ref.into_inner()
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

impl IncrGlyphPackBuilder {
    pub fn finalize_delta(&self) -> (FontPack, Vec<(DefId, GlyphItem)>) {
        let fonts = std::mem::take(self.new_fonts.lock().deref_mut());
        let glyphs = std::mem::take(self.new_glyphs.lock().deref_mut());
        (fonts, glyphs)
    }
}

pub trait FontIndice<'m> {
    fn get_font(&self, value: &FontRef) -> Option<&'m FontItem>;
}

pub trait GlyphIndice<'m> {
    fn get_glyph(&self, value: &GlyphRef) -> Option<&'m GlyphItem>;
}

pub trait BuildGlyph {
    fn build_font(&mut self, font: &Font) -> FontRef;
    fn build_glyph(&mut self, glyph: &GlyphItem) -> GlyphRef;
}

pub trait GlyphHashStablizer {
    fn stablize_hash(&mut self, glyph: &GlyphRef) -> Fingerprint;
}

/// Source mapping
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum SourceMappingNode {
    Group(Arc<[u64]>),
    Text(SpanId),
    Image(SpanId),
    Shape(SpanId),
    Page(u64),
}

// todo: remove this function
pub fn flatten_glyphs(
    repr: impl IntoIterator<Item = (DefId, GlyphItem)>,
) -> Vec<(DefId, FlatGlyphItem)> {
    let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
    let glyph_lower_builder = Glyph2VecPass::new(&glyph_provider, true);

    repr.into_iter()
        .map(|(font_id, glyph)| {
            let glyph = glyph_lower_builder.glyph(&glyph);
            glyph
                .map(|t| {
                    let t = match t {
                        GlyphItem::Image(i) => FlatGlyphItem::Image(i),
                        GlyphItem::Outline(p) => FlatGlyphItem::Outline(p),
                        GlyphItem::None => FlatGlyphItem::None,
                        _ => unreachable!(),
                    };

                    (font_id, t)
                })
                .unwrap_or_else(|| (font_id, FlatGlyphItem::None))
        })
        .collect::<Vec<_>>()
}

pub fn serialize_doc(doc: MultiVecDocument) -> Vec<u8> {
    let flatten_module = FlatModule::new(vec![
        ModuleMetadata::Item(ItemPack(doc.module.items.into_iter().collect())),
        ModuleMetadata::Font(Arc::new(doc.module.fonts.into())),
        ModuleMetadata::Glyph(Arc::new(flatten_glyphs(doc.module.glyphs).into())),
        ModuleMetadata::Layout(Arc::new(doc.layouts)),
    ]);

    flatten_module.to_bytes()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::hash::Fingerprint;
    use crate::vector::geom::{Axes, Scalar};

    use crate::vector::ir::{Image, ImageItem};

    /// Test image serialization.
    #[test]
    fn test_image_serialization() {
        let img = ImageItem {
            image: Arc::new(Image {
                data: vec![0, 1, 2, 3],
                format: "png".into(),
                size: Axes::new(10, 10),
                alt: None,
                hash: Fingerprint::from_pair(0xdeadbeef, 0),
            }),
            size: Axes::new(Scalar(10.0), Scalar(10.0)),
        };

        // Or you can customize your serialization for better performance
        // and compatibility with #![no_std] environments
        use rkyv::ser::{serializers::AllocSerializer, Serializer};

        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(&img).unwrap();
        let bytes = serializer.into_serializer().into_inner();

        let ret = bytes.into_vec();
        assert_eq!("00010203706e6700f8ffffff04000000f4ffffff030000000a0000000a000000efbeadde000000000000000000000000000000000000000000000000000000000000204100002041c0ffffff", hex::encode(ret));
    }
}
