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
use std::sync::Arc;

mod color;
mod compose;
pub mod geom;
pub mod layout;
mod meta;
pub mod module;
mod preludes;
mod primitives;
mod text;
mod visualize;

pub use color::*;
pub use compose::*;
pub use geom::*;
pub use layout::*;
pub use meta::*;
pub use module::*;
pub use primitives::*;
pub use text::*;
pub use visualize::*;

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

use crate::{hash::Fingerprint, TakeAs};

pub use crate::ImmutStr;

use super::pass::IncrGlyph2VecPass;
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

pub type GlyphPackBuilder = Glyph2VecPass;
pub type IncrGlyphPackBuilder = IncrGlyph2VecPass;

pub trait FontIndice<'m> {
    fn get_font(&self, value: &FontRef) -> Option<&'m FontItem>;
}
pub trait ItemIndice<'m> {
    fn get_item(&self, value: &Fingerprint) -> Option<&'m VecItem>;
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

pub fn serialize_doc(doc: MultiVecDocument) -> Vec<u8> {
    let flatten_module = FlatModule::new(vec![
        ModuleMetadata::Item(ItemPack(doc.module.items.into_iter().collect())),
        ModuleMetadata::Font(Arc::new(doc.module.fonts.into())),
        ModuleMetadata::Glyph(Arc::new(doc.module.glyphs.into())),
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
