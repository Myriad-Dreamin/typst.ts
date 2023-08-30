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

use std::{collections::HashMap, ops::Index, sync::Arc};

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

use crate::{
    font::{FontGlyphProvider, GlyphProvider},
    hash::{Fingerprint, FingerprintBuilder},
    vector::GlyphLowerBuilder,
};

use super::{
    geom::{Abs, Point, Size},
    ir::{
        AbsoluteRef, BuildGlyph, DefId, FontRef, GlyphItem, GlyphPackBuilder, GlyphRef,
        ImageGlyphItem, ImageItem, ImmutStr, LinkItem, OutlineGlyphItem, PathItem, Scalar, SpanId,
        SvgItem, TextShape, TransformItem,
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
    Group(GroupRef),
}

/// Flatten text item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FlatTextItem {
    pub content: Arc<FlatTextItemContent>,
    pub shape: Arc<TextShape>,
}

/// The content metadata of a [`FlatTextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FlatTextItemContent {
    pub content: ImmutStr,
    pub glyphs: Arc<[(Abs, Abs, GlyphRef)]>,
}

/// Reference a glyph item in a more friendly format to compress and store
/// information. The glyphs are locally stored in the svg module.
/// With a glyph reference, we can get both the font metric and the glyph data.
/// The `font_hash` is to let it safe to be cached.
/// By estimation, <https://stackoverflow.com/a/29628053/9323228>
/// If the hash algorithm for `font_hash` is good enough.
/// When you have about 500 fonts (in windows), the collision rate is about:
/// ```plain
/// p(n = 500, d = 2^32) = 1 - exp(-n^2/(2d))
///   = 1 - exp(-500^2/(2*(2^32))) = 0.0000291034
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FlatFontItem {
    /// The hash of the font to avoid global collision.
    pub fingerprint: Fingerprint,
    /// The inlined hash of the font to avoid local collision.
    pub hash: u32,
}

/// The glyph item definition with all of variants of [`GlyphItem`] other than
/// [`GlyphItem::Raw`], hence it is serializable.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum FlatGlyphItem {
    Image(Arc<ImageGlyphItem>),
    Outline(Arc<OutlineGlyphItem>),
}

impl From<FlatGlyphItem> for GlyphItem {
    fn from(item: FlatGlyphItem) -> Self {
        match item {
            FlatGlyphItem::Image(item) => GlyphItem::Image(item),
            FlatGlyphItem::Outline(item) => GlyphItem::Outline(item),
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
    pub items: Vec<FlatFontItem>,
    pub incremental_base: usize,
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
        let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider);

        let items = iter
            .into_iter()
            .flat_map(|(glyph, glyph_id)| {
                let glyph = glyph_lower_builder.lower_glyph(&glyph);
                glyph.map(|t| {
                    let t = match t {
                        GlyphItem::Image(i) => FlatGlyphItem::Image(i),
                        GlyphItem::Outline(p) => FlatGlyphItem::Outline(p),
                        _ => unreachable!(),
                    };

                    (DefId(glyph_id.1.idx as u64), t)
                })
            })
            .collect::<Vec<_>>();

        Self {
            items,
            incremental_base: 0,
        }
    }
}

/// Describing reference to a page
#[derive(Debug, Clone)]
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

/// Describing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
#[repr(C)]
pub enum LayoutRegionNode {
    // next indirection
    Indirect(usize),
    // flat page layout
    Pages(Arc<(Vec<PageMetadata>, Vec<Page>)>),
    // source mapping node per page
    SourceMapping(Arc<(Vec<PageMetadata>, Vec<SourceMappingNode>)>),
}

impl LayoutRegionNode {
    pub fn new_pages(pages: Vec<Page>) -> Self {
        Self::Pages(Arc::new((Default::default(), pages)))
    }

    pub fn new_source_mapping(source_mapping: Vec<SourceMappingNode>) -> Self {
        Self::SourceMapping(Arc::new((Default::default(), source_mapping)))
    }
}

/// Describing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LayoutRegionRepr<T> {
    pub kind: ImmutStr,
    pub layouts: Vec<(T, LayoutRegionNode)>,
}

/// Describing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum LayoutRegion {
    ByScalar(LayoutRegionRepr<Scalar>),
    ByStr(LayoutRegionRepr<ImmutStr>),
}

impl LayoutRegion {
    pub fn new_single(layout: LayoutRegionNode) -> Self {
        Self::ByScalar(LayoutRegionRepr {
            kind: "_".into(),
            layouts: vec![(Default::default(), layout)],
        })
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::ByScalar(v) => v.layouts.is_empty(),
            Self::ByStr(v) => v.layouts.is_empty(),
        }
    }
}

impl Index<usize> for LayoutRegion {
    type Output = LayoutRegionNode;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Self::ByScalar(v) => &v.layouts[index].1,
            Self::ByStr(v) => &v.layouts[index].1,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LayoutSourceMapping(pub LayoutRegion);

impl Default for LayoutSourceMapping {
    fn default() -> Self {
        Self::new_single(Default::default())
    }
}

impl LayoutSourceMapping {
    pub fn new_single(source_mapping: Vec<SourceMappingNode>) -> Self {
        Self(LayoutRegion::new_single(
            LayoutRegionNode::new_source_mapping(source_mapping),
        ))
    }
}

pub type ItemMap = rustc_hash::FxHashMap<Fingerprint, FlatSvgItem>;

pub type RefItemMap = HashMap<Fingerprint, (u64, FlatSvgItem)>;

pub trait ToItemMap {
    fn to_item_map(self) -> ItemMap;
}

impl ToItemMap for RefItemMap {
    fn to_item_map(self) -> ItemMap {
        self.into_iter().map(|(k, (_, v))| (k, v)).collect::<_>()
    }
}

/// Trait of a streaming representation of a module.
pub trait ModuleStream {
    fn items(&self) -> ItemPack;
    fn layouts(&self) -> LayoutRegion;
    fn glyphs(&self) -> Vec<(DefId, FlatGlyphItem)>;
    fn gc_items(&self) -> Option<Vec<Fingerprint>> {
        // never gc items
        None
    }
}

/// A finished module that stores all the svg items.
/// The svg items shares the underlying data.
/// The svg items are flattened and ready to be serialized.
#[derive(Debug, Default)]
pub struct Module {
    pub glyphs: Vec<(DefId, GlyphItem)>,
    pub items: ItemMap,
    pub source_mapping: Vec<SourceMappingNode>,
}

impl Module {
    /// Get a glyph item by its stable ref.
    pub fn get_glyph(&self, id: &AbsoluteRef) -> Option<&GlyphItem> {
        self.glyphs.get(id.id.0 as usize).map(|(_, item)| item)
    }

    /// Get a svg item by its stable ref.
    pub fn get_item(&self, id: &Fingerprint) -> Option<&FlatSvgItem> {
        self.items.get(id)
    }

    pub fn merge_delta(&mut self, v: impl ModuleStream) {
        let item_pack: ItemPack = v.items();
        let glyphs = v.glyphs();

        if let Some(gc_items) = v.gc_items() {
            for id in gc_items {
                self.items.remove(&id);
            }
        }

        self.items.extend(item_pack.0);
        self.glyphs
            .extend(glyphs.into_iter().map(|(id, item)| (id, item.into())));
    }
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
    Glyph(Arc<GlyphPack>),
    Layout(Arc<LayoutRegion>),
}

const _: () = assert!(core::mem::size_of::<ModuleMetadata>() == 32);

#[repr(usize)]
enum MetaIndices {
    Version,
    SourceMapping,
    PageSourceMapping,
    GarbageCollection,
    Item,
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
            for m in &self.metadata {
                if let ModuleMetadata::Item(v) = m {
                    sz = v.0.len();
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

    fn layouts(&self) -> LayoutRegion {
        // self.layouts.clone()
        todo!()
    }

    fn glyphs(&self) -> Vec<(DefId, FlatGlyphItem)> {
        // self.glyphs.clone()
        todo!()
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

pub type LayoutElem = (
    /* layout width */ Abs,
    /* layout pages */ Vec<Page>,
);

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
    pub layouts: LayoutRegion,
}

impl Default for MultiSvgDocument {
    fn default() -> Self {
        let pages = LayoutRegionNode::new_pages(Default::default());
        Self {
            module: Default::default(),
            layouts: LayoutRegion::new_single(pages),
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
        self.layouts = v.layouts();
        self.module.merge_delta(v);
    }
}

/// Intermediate representation of a incompleted svg item.
pub struct ModuleBuilderImpl<const ENABLE_REF_CNT: bool = false> {
    pub glyphs: GlyphPackBuilder,
    pub items: HashMap<Fingerprint, (u64, FlatSvgItem)>,
    pub source_mapping: Vec<SourceMappingNode>,
    pub source_mapping_buffer: Vec<u64>,

    fingerprint_builder: FingerprintBuilder,

    /// See `typst_ts_svg_exporter::ExportFeature`.
    pub should_attach_debug_info: bool,

    pub lifetime: u64,
    pub incr_glyphs: Vec<u64>,
}

pub type ModuleBuilder = ModuleBuilderImpl</* ENABLE_REF_CNT */ false>;
pub type IncrModuleBuilder = ModuleBuilderImpl</* ENABLE_REF_CNT */ true>;

impl<const ENABLE_REF_CNT: bool> Default for ModuleBuilderImpl<ENABLE_REF_CNT> {
    fn default() -> Self {
        Self {
            lifetime: 0,
            glyphs: Default::default(),
            items: Default::default(),
            source_mapping: Default::default(),
            source_mapping_buffer: Default::default(),
            fingerprint_builder: Default::default(),
            incr_glyphs: Default::default(),
            should_attach_debug_info: false,
        }
    }
}

impl<const ENABLE_REF_CNT: bool> BuildGlyph for ModuleBuilderImpl<ENABLE_REF_CNT> {
    fn build_glyph(&mut self, glyph: &GlyphItem) -> GlyphRef {
        // if ENABLE_REF_CNT {
        //     self.incr_glyphs.push(self.lifetime);
        // }
        todo!() // todo: font

        // self.glyphs.build_glyph(glyph)

        // let abs_ref = AbsoluteRef { fingerprint, id };
        // self.glyphs.insert(glyph.clone(), abs_ref.clone());
        // abs_ref
    }
}

impl<const ENABLE_REF_CNT: bool> ModuleBuilderImpl<ENABLE_REF_CNT> {
    pub fn reset(&mut self) {
        self.source_mapping.clear();
        self.source_mapping_buffer.clear();
    }

    // todo: remove GlyphMapping (used by v1)
    pub fn finalize_ref(&self) -> Module {
        Module {
            glyphs: self.glyphs.finalize(),
            items: self.items.clone().to_item_map(),
            source_mapping: self.source_mapping.clone(),
        }
    }

    // todo: remove GlyphMapping (used by v1)
    pub fn finalize(self) -> Module {
        Module {
            glyphs: self.glyphs.finalize(),
            items: self.items.to_item_map(),
            source_mapping: self.source_mapping,
        }
    }

    pub fn build(&mut self, item: SvgItem) -> Fingerprint {
        let resolved_item = match item {
            SvgItem::Image((image, span_id)) => {
                if self.should_attach_debug_info {
                    let sm_id = self.source_mapping.len() as u64;
                    self.source_mapping.push(SourceMappingNode::Image(span_id));
                    self.source_mapping_buffer.push(sm_id);
                }

                FlatSvgItem::Image(image)
            }
            SvgItem::Path((path, span_id)) => {
                if self.should_attach_debug_info {
                    let sm_id = self.source_mapping.len() as u64;
                    self.source_mapping.push(SourceMappingNode::Shape(span_id));
                    self.source_mapping_buffer.push(sm_id);
                }

                FlatSvgItem::Path(path)
            }
            SvgItem::Link(link) => FlatSvgItem::Link(link),
            SvgItem::Text(text) => {
                let glyphs = text
                    .content
                    .glyphs
                    .iter()
                    .cloned()
                    .map(|(offset, advance, glyph)| (offset, advance, self.build_glyph(&glyph)))
                    .collect::<Arc<_>>();
                let shape = text.shape.clone();
                let content = text.content.content.clone();

                if self.should_attach_debug_info {
                    let sm_id = self.source_mapping.len() as u64;
                    self.source_mapping
                        .push(SourceMappingNode::Text(text.content.span_id));
                    self.source_mapping_buffer.push(sm_id);
                }

                FlatSvgItem::Text(FlatTextItem {
                    content: Arc::new(FlatTextItemContent { content, glyphs }),
                    shape,
                })
            }
            SvgItem::Transformed(transformed) => {
                let item = &transformed.1;
                let item_id = self.build(*item.clone());
                let transform = transformed.0.clone();

                FlatSvgItem::Item(TransformedRef(transform, item_id))
            }
            SvgItem::Group(group) => {
                let t = if self.should_attach_debug_info {
                    Some(self.source_mapping_buffer.len())
                } else {
                    None
                };
                let items = group
                    .0
                    .iter()
                    .map(|(point, item)| (*point, self.build(item.clone())))
                    .collect::<Vec<_>>();

                if self.should_attach_debug_info {
                    let sm_start = unsafe { t.unwrap_unchecked() };
                    let sm_range = self
                        .source_mapping_buffer
                        .splice(sm_start..self.source_mapping_buffer.len(), []);

                    let sm_id = self.source_mapping.len() as u64;
                    self.source_mapping
                        .push(SourceMappingNode::Group(sm_range.collect()));
                    self.source_mapping_buffer.push(sm_id);
                }
                FlatSvgItem::Group(GroupRef(items.into()))
            }
        };

        let fingerprint = self.fingerprint_builder.resolve(&resolved_item);

        if let Some(pos) = self.items.get_mut(&fingerprint) {
            if ENABLE_REF_CNT && pos.0 != self.lifetime {
                pos.0 = self.lifetime - 1;
            }
            return fingerprint;
        }

        if ENABLE_REF_CNT {
            self.items
                .insert(fingerprint, (self.lifetime, resolved_item));
        } else {
            self.items.insert(fingerprint, (0, resolved_item));
        }
        fingerprint
    }
}

impl IncrModuleBuilder {
    /// Increment the lifetime of the module.
    /// It increments by 2 which is used to distinguish between the
    /// retained items and the new items.
    /// Assuming that the old lifetime is 'l,
    /// the retained and new lifetime will be 'l + 1 and 'l + 2, respectively.
    pub fn increment_lifetime(&mut self) {
        self.lifetime += 2;
    }

    /// Perform garbage collection with given threshold.
    pub fn gc(&mut self, threshold: u64) -> Vec<Fingerprint> {
        let mut gc_items = vec![];

        // a threshold is set by current lifetime subtracted by the given threshold.
        // It uses saturating_sub to prevent underflow (u64).
        let threshold = self.lifetime.saturating_sub(threshold);

        self.items.retain(|k, v| {
            if v.0 < threshold {
                gc_items.push(*k);
                false
            } else {
                true
            }
        });

        gc_items
    }

    /// Finalize modules containing new svg items.
    pub fn finalize_delta(&mut self) -> Module {
        // fliter glyphs by lifetime
        let glyphs = {
            // let glyphs = self.glyphs.iter();
            // let glyphs =
            //     glyphs.filter(|e| self.incr_glyphs[e.1 .0.glyph_idx as usize] ==
            // self.lifetime); let glyphs = glyphs.map(|(x, y)| (x.clone(),
            // y.clone()));

            // GlyphPackBuilder::finalize(glyphs)
            self.glyphs.finalize()
        };

        // fliter glyphs by lifetime
        let items = {
            let items = self.items.iter();
            let items = items.filter(|(_, e)| e.0 == self.lifetime);
            let items = items.map(|(f, (_, i))| (*f, i.clone()));

            ItemMap::from_iter(items)
        };

        Module {
            glyphs,
            items,
            source_mapping: self.source_mapping.clone(),
        }
    }
}

// todo: remove this function
pub fn flatten_glyphs(
    repr: impl IntoIterator<Item = (DefId, GlyphItem)>,
) -> Vec<(DefId, FlatGlyphItem)> {
    let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
    let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider);

    repr.into_iter()
        .flat_map(|(glyph_id, glyph)| {
            let glyph = glyph_lower_builder.lower_glyph(&glyph);
            glyph.map(|t| {
                let t = match t {
                    GlyphItem::Image(i) => FlatGlyphItem::Image(i),
                    GlyphItem::Outline(p) => FlatGlyphItem::Outline(p),
                    _ => unreachable!(),
                };

                (glyph_id, t)
            })
        })
        .collect::<Vec<_>>()
}

pub fn serialize_doc(doc: MultiSvgDocument) -> Vec<u8> {
    // let flatten_module = FlatModule {
    //     magic: *b"tsvr\x00\x00\x00\x00",
    //     metadata: vec![],
    //     // item_pack: ItemPack(doc.module.items.into_iter().collect()),
    //     // glyphs: flatten_glyphs(glyph_mapping),
    //     // layouts: doc.layouts,
    //     meta_indices: Default::default(),
    // };

    // flatten_module.to_bytes()
    todo!()
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
