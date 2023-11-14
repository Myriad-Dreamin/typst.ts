//! Flat intermediate representation of svg items.
//!
//! SvgDoc and Module Relation:
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
//! │[`SvgDocument`]│◄───────────────┤[`MultiSvgDocument`]│
//! └───────────────┘                └────────────────────┘

use std::sync::Arc;

mod module;
pub use module::*;

mod layout;
pub use layout::*;

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

use crate::{
    font::{FontGlyphProvider, GlyphProvider},
    hash::Fingerprint,
    vector::GlyphLowerBuilder,
    TakeAs,
};

use super::{
    geom::{Abs, Point, Size},
    ir::{
        DefId, FontItem, FontRef, GlyphItem, GlyphRef, GradientItem, ImageGlyphItem, ImageItem,
        ImmutStr, LinkItem, OutlineGlyphItem, PathItem, SpanId, TextShape, TransformItem,
    },
};

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

/// Flatten svg item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum FlatSvgItem {
    None,
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(FlatTextItem),
    Item(TransformedRef),
    Group(GroupRef, Option<Size>),
    Gradient(GradientItem),
}

/// Flatten text item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FlatTextItem {
    pub font: FontRef,
    pub content: Arc<FlatTextItemContent>,
    pub shape: Arc<TextShape>,
}

impl FlatTextItem {
    pub fn render_glyphs(&self, upem: Abs, consume_glyph: impl FnMut(Abs, &GlyphRef)) -> Abs {
        self.shape
            .render_glyphs(upem, self.content.glyphs.iter(), consume_glyph)
    }
}

/// The content metadata of a [`FlatTextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FlatTextItemContent {
    pub content: ImmutStr,
    pub glyphs: Arc<[(Abs, Abs, GlyphRef)]>,
}

/// The glyph item definition with all of variants of [`GlyphItem`] other than
/// [`GlyphItem::Raw`], hence it is serializable.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum FlatGlyphItem {
    None,
    Image(Arc<ImageGlyphItem>),
    Outline(Arc<OutlineGlyphItem>),
}

impl From<FlatGlyphItem> for GlyphItem {
    fn from(item: FlatGlyphItem) -> Self {
        match item {
            FlatGlyphItem::Image(item) => GlyphItem::Image(item),
            FlatGlyphItem::Outline(item) => GlyphItem::Outline(item),
            FlatGlyphItem::None => GlyphItem::None,
        }
    }
}

/// Flatten transform item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct TransformedRef(pub TransformItem, pub Fingerprint);

/// Flatten group item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct GroupRef(pub Arc<[(Point, Fingerprint)]>);

/// Flatten mapping fingerprints to svg items.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct ItemPack(pub Vec<(Fingerprint, FlatSvgItem)>);

/// Flatten mapping fingerprints to glyph items.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FontPack {
    pub items: Vec<FontItem>,
    pub incremental_base: usize,
}

impl From<Vec<FontItem>> for FontPack {
    fn from(items: Vec<FontItem>) -> Self {
        Self {
            items,
            incremental_base: 0,
        }
    }
}

/// Flatten mapping fingerprints to glyph items.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct GlyphPack {
    pub items: Vec<(DefId, FlatGlyphItem)>,
    pub incremental_base: usize,
}

impl From<Vec<(DefId, FlatGlyphItem)>> for GlyphPack {
    fn from(items: Vec<(DefId, FlatGlyphItem)>) -> Self {
        Self {
            items,
            incremental_base: 0,
        }
    }
}

impl FromIterator<(GlyphItem, (GlyphRef, FontRef))> for GlyphPack {
    fn from_iter<T: IntoIterator<Item = (GlyphItem, (GlyphRef, FontRef))>>(iter: T) -> Self {
        let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
        let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider, true);

        let items = iter
            .into_iter()
            .map(|(glyph, glyph_id)| {
                let glyph = glyph_lower_builder.lower_glyph(&glyph);
                glyph
                    .map(|t| {
                        let t = match t {
                            GlyphItem::Image(i) => FlatGlyphItem::Image(i),
                            GlyphItem::Outline(p) => FlatGlyphItem::Outline(p),
                            _ => unreachable!(),
                        };

                        (DefId(glyph_id.1.idx as u64), t)
                    })
                    .unwrap_or_else(|| (DefId(glyph_id.1.idx as u64), FlatGlyphItem::None))
            })
            .collect::<Vec<_>>();

        Self {
            items,
            incremental_base: 0,
        }
    }
}

/// Describing reference to a page
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Page {
    /// Unique hash to content
    pub content: Fingerprint,
    /// Page size for cropping content
    pub size: Size,
}

/// metadata that can be attached to a module.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
#[repr(C, align(32))]
pub enum PageMetadata {
    GarbageCollection(Vec<Fingerprint>),
    Item(ItemPack),
    Glyph(Arc<GlyphPack>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct BuildInfo {
    pub version: ImmutStr,
    pub compiler: ImmutStr,
}

/// metadata that can be attached to a module.
#[derive(Debug, Clone)]
#[repr(C, align(32))]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ModuleMetadata {
    BuildVersion(Arc<BuildInfo>),
    SourceMappingData(Vec<SourceMappingNode>),
    PageSourceMapping(Arc<LayoutSourceMapping>),
    GarbageCollection(Vec<Fingerprint>),
    Item(ItemPack),
    Font(Arc<FontPack>),
    Glyph(Arc<GlyphPack>),
    Layout(Arc<Vec<LayoutRegion>>),
}

const _: () = assert!(core::mem::size_of::<ModuleMetadata>() == 32);

#[repr(usize)]
#[allow(dead_code)]
enum MetaIndices {
    Version,
    SourceMapping,
    PageSourceMapping,
    GarbageCollection,
    Item,
    Font,
    Glyph,
    Layout,
    Max,
}

const META_INDICES_MAX: usize = MetaIndices::Max as usize;

/// Flatten module so that it can be serialized.
#[derive(Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FlatModule {
    pub magic: [u8; 8],
    pub metadata: Vec<ModuleMetadata>,

    #[with(rkyv::with::Skip)]
    meta_indices: [once_cell::sync::OnceCell<usize>; META_INDICES_MAX],
    // pub item_pack: ItemPack,
    // pub glyphs: Vec<(AbsoluteRef, FlatGlyphItem)>,
    // pub layouts: Vec<(Abs, Vec<(Fingerprint, Size)>)>,
}

impl Default for FlatModule {
    fn default() -> Self {
        Self {
            magic: *b"tsvr\x00\x00\x00\x00",
            metadata: vec![],
            meta_indices: Default::default(),
        }
    }
}

#[cfg(feature = "rkyv")]
impl FlatModule {
    pub fn new(metadata: Vec<ModuleMetadata>) -> Self {
        Self {
            metadata,
            ..Default::default()
        }
    }

    pub fn to_bytes(self: &FlatModule) -> Vec<u8> {
        // Or you can customize your serialization for better performance
        // and compatibility with #![no_std] environments
        use rkyv::ser::{serializers::AllocSerializer, Serializer};

        let mut serializer = AllocSerializer::<0>::default();
        serializer.serialize_value(self).unwrap();
        let bytes = serializer.into_serializer().into_inner();

        bytes.into_vec()
    }
}

// todo: for archived module.
// todo: zero copy
#[cfg(feature = "rkyv")]
impl ModuleStream for &FlatModule {
    fn items(&self) -> ItemPack {
        // cache the index
        let sz = &self.meta_indices[MetaIndices::Item as usize];
        let sz = sz.get_or_init(|| {
            let mut sz = usize::MAX; // will panic if not found
            for (idx, m) in self.metadata.iter().enumerate() {
                if let ModuleMetadata::Item(_) = m {
                    sz = idx;
                    break;
                }
            }
            sz
        });

        // get the item pack
        let m = &self.metadata[*sz];
        if let ModuleMetadata::Item(v) = m {
            v.clone()
        } else {
            unreachable!()
        }
    }

    fn layouts(&self) -> Arc<Vec<LayoutRegion>> {
        // cache the index
        let sz = &self.meta_indices[MetaIndices::Layout as usize];
        let sz = sz.get_or_init(|| {
            let mut sz = usize::MAX; // will panic if not found
            for (idx, m) in self.metadata.iter().enumerate() {
                if let ModuleMetadata::Layout(_) = m {
                    sz = idx;
                    break;
                }
            }
            sz
        });

        // get the item pack
        let m = &self.metadata[*sz];
        if let ModuleMetadata::Layout(v) = m {
            v.clone()
        } else {
            unreachable!()
        }
    }

    fn fonts(&self) -> Arc<FontPack> {
        // cache the index
        let sz = &self.meta_indices[MetaIndices::Font as usize];
        let sz = sz.get_or_init(|| {
            let mut sz = usize::MAX; // will panic if not found
            for (idx, m) in self.metadata.iter().enumerate() {
                if let ModuleMetadata::Font(_) = m {
                    sz = idx;
                    break;
                }
            }
            sz
        });

        // get the item pack
        let m = &self.metadata[*sz];
        if let ModuleMetadata::Font(v) = m {
            v.clone()
        } else {
            unreachable!()
        }
    }

    fn glyphs(&self) -> Arc<GlyphPack> {
        // cache the index
        let sz = &self.meta_indices[MetaIndices::Glyph as usize];
        let sz = sz.get_or_init(|| {
            let mut sz = usize::MAX; // will panic if not found
            for (idx, m) in self.metadata.iter().enumerate() {
                if let ModuleMetadata::Glyph(_) = m {
                    sz = idx;
                    break;
                }
            }
            sz
        });

        // get the item pack
        let m = &self.metadata[*sz];
        if let ModuleMetadata::Glyph(v) = m {
            v.clone()
        } else {
            unreachable!()
        }
    }

    fn gc_items(&self) -> Option<Vec<Fingerprint>> {
        for m in &self.metadata {
            if let ModuleMetadata::GarbageCollection(v) = m {
                return Some(v.clone());
            }
        }
        None
    }
}

/// Module with page references of a [`typst::doc::Document`].
pub struct SvgDocument {
    /// module containing all of the data related to this document.
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub pages: Vec<Page>,
}

/// Module with multiple documents in a module [`typst::doc::Document`].
pub struct MultiSvgDocument {
    /// module containing all of the data related to this document.
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub layouts: Vec<LayoutRegion>,
}

impl Default for MultiSvgDocument {
    fn default() -> Self {
        let pages = LayoutRegionNode::new_pages(Default::default());
        Self {
            module: Default::default(),
            layouts: vec![LayoutRegion::new_single(pages)],
        }
    }
}

impl MultiSvgDocument {
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

// todo: remove this function
pub fn flatten_glyphs(
    repr: impl IntoIterator<Item = (DefId, GlyphItem)>,
) -> Vec<(DefId, FlatGlyphItem)> {
    let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
    let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider, false);

    repr.into_iter()
        .map(|(font_id, glyph)| {
            let glyph = glyph_lower_builder.lower_glyph(&glyph);
            glyph
                .map(|t| {
                    let t = match t {
                        GlyphItem::Image(i) => FlatGlyphItem::Image(i),
                        GlyphItem::Outline(p) => FlatGlyphItem::Outline(p),
                        _ => unreachable!(),
                    };

                    (font_id, t)
                })
                .unwrap_or_else(|| (font_id, FlatGlyphItem::None))
        })
        .collect::<Vec<_>>()
}

pub fn serialize_doc(doc: MultiSvgDocument) -> Vec<u8> {
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
