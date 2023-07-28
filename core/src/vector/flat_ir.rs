use std::{collections::HashMap, sync::Arc};

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
        AbsoluteRef, DefId, GlyphItem, GlyphMapping, GlyphPackBuilder, ImageGlyphItem, ImageItem,
        ImmutStr, LinkItem, OutlineGlyphItem, PathItem, SpanId, SvgItem, TextShape, TransformItem,
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
    pub glyphs: Arc<[(Abs, Abs, AbsoluteRef)]>,
}

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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct ItemPack(pub Vec<(Fingerprint, FlatSvgItem)>);

pub type ItemMap = rustc_hash::FxHashMap<Fingerprint, FlatSvgItem>;

/// A finished module that stores all the svg items.
/// The svg items shares the underlying data.
/// The svg items are flattened and ready to be serialized.
#[derive(Debug, Default)]
pub struct Module {
    pub glyphs: Vec<(AbsoluteRef, GlyphItem)>,
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
}

pub type Pages = Vec<(Fingerprint, Size)>;
pub type LayoutElem = (Abs, Pages);

/// Module with page references of a [`typst::doc::Document`].
pub struct SvgDocument {
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub pages: Pages,
}

/// Module with multiple documents in a module [`typst::doc::Document`].
#[derive(Default)]
pub struct MultiSvgDocument {
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub layouts: Vec<(Abs, Pages)>,
}

#[cfg(feature = "rkyv")]
pub mod stream {
    use super::{ArchivedSerializedModule, SerializedModule};
    use rkyv::de::deserializers::SharedDeserializeMap;
    use rkyv::{AlignedVec, Deserialize};

    enum RkyvStreamData<'a> {
        Aligned(&'a [u8]),
        Unaligned(AlignedVec),
    }

    impl<'a> AsRef<[u8]> for RkyvStreamData<'a> {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            match self {
                Self::Aligned(v) => v,
                Self::Unaligned(v) => v.as_slice(),
            }
        }
    }

    pub struct SvgDocumentStream<'a> {
        data: RkyvStreamData<'a>,
    }

    impl<'a> SvgDocumentStream<'a> {
        pub fn from_slice(v: &'a [u8]) -> Self {
            let v = if (v.as_ptr() as usize) % AlignedVec::ALIGNMENT != 0 {
                let mut aligned = AlignedVec::with_capacity(v.len());
                aligned.extend_from_slice(v);
                RkyvStreamData::Unaligned(aligned)
            } else {
                RkyvStreamData::Aligned(v)
            };

            Self { data: v }
        }

        pub fn checkout(&self) -> &ArchivedSerializedModule {
            rkyv::check_archived_root::<SerializedModule>(self.data.as_ref()).unwrap()
        }

        pub fn checkout_owned(&self) -> SerializedModule {
            let v = self.checkout();
            let mut dmap = SharedDeserializeMap::default();
            v.deserialize(&mut dmap).unwrap()
        }
    }
}

pub trait SvgDocumentStreamView {
    fn get_items(&self) -> ItemPack;
    fn get_layouts(&self) -> Vec<(Abs, Pages)>;
    fn get_glyphs(&self) -> Vec<(AbsoluteRef, FlatGlyphItem)>;
    fn get_gc_items(&self) -> Option<Vec<Fingerprint>>;
}

// todo: for archived module.
// todo: zero copy
#[cfg(feature = "rkyv")]
impl SvgDocumentStreamView for &SerializedModule {
    fn get_items(&self) -> ItemPack {
        self.item_pack.clone()
    }

    fn get_layouts(&self) -> Vec<(Abs, Pages)> {
        self.layouts.clone()
    }

    fn get_glyphs(&self) -> Vec<(AbsoluteRef, FlatGlyphItem)> {
        self.glyphs.clone()
    }

    fn get_gc_items(&self) -> Option<Vec<Fingerprint>> {
        for m in &self.metadata {
            if let ModuleMetadata::GarbageCollection(v) = m {
                return Some(v.clone());
            }
        }
        None
    }
}

impl MultiSvgDocument {
    pub fn merge_delta(&mut self, v: impl SvgDocumentStreamView) {
        let item_pack: ItemPack = v.get_items();
        let layouts = v.get_layouts();
        let glyphs = v.get_glyphs();

        self.layouts = layouts;

        if let Some(gc_items) = v.get_gc_items() {
            for id in gc_items {
                self.module.items.remove(&id);
            }
        }

        self.module.items.extend(item_pack.0.into_iter());
        self.module
            .glyphs
            .extend(glyphs.into_iter().map(|(id, item)| (id, item.into())));
    }

    pub fn from_slice(v: &[u8]) -> Self {
        let data = stream::SvgDocumentStream::from_slice(v);

        let mut res: Self = Default::default();
        res.merge_delta(&data.checkout_owned());
        res
    }
}

/// Intermediate representation of a incompleted svg item.
pub struct ModuleBuilderImpl<const ENABLE_REF_CNT: bool = false> {
    pub glyphs: GlyphMapping,
    pub items: HashMap<Fingerprint, (u64, FlatSvgItem)>,
    pub source_mapping: Vec<SourceMappingNode>,
    pub source_mapping_buffer: Vec<u64>,

    fingerprint_builder: FingerprintBuilder,

    /// See `typst_ts_svg_exporter::ExportFeature`.
    pub should_attach_debug_info: bool,

    pub lifetime: u64,
    pub incr_glyphs: Vec<u64>,
}

pub type ModuleBuilder = ModuleBuilderImpl<false>;
pub type IncrModuleBuilder = ModuleBuilderImpl<true>;

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

impl<const ENABLE_REF_CNT: bool> ModuleBuilderImpl<ENABLE_REF_CNT> {
    pub fn reset(&mut self) {
        self.source_mapping.clear();
        self.source_mapping_buffer.clear();
    }

    pub fn finalize_ref(&self) -> (Module, GlyphMapping) {
        let glyphs = GlyphPackBuilder::finalize(self.glyphs.clone());

        let module = Module {
            glyphs,
            items: ItemMap::from_iter(self.items.clone().into_iter().map(|(f, (_, i))| (f, i))),
            source_mapping: self.source_mapping.clone(),
        };

        (module, self.glyphs.clone())
    }

    pub fn finalize(self) -> (Module, GlyphMapping) {
        let glyphs = GlyphPackBuilder::finalize(self.glyphs.clone());

        let module = Module {
            glyphs,
            items: ItemMap::from_iter(self.items.clone().into_iter().map(|(f, (_, i))| (f, i))),
            source_mapping: self.source_mapping,
        };

        (module, self.glyphs)
    }

    pub fn build_glyph(&mut self, glyph: &GlyphItem) -> AbsoluteRef {
        if let Some(id) = self.glyphs.get(glyph) {
            return id.clone();
        }

        let id = DefId(self.glyphs.len() as u64);

        let fingerprint = self.fingerprint_builder.resolve(glyph);
        let abs_ref = AbsoluteRef { fingerprint, id };
        self.glyphs.insert(glyph.clone(), abs_ref.clone());
        if ENABLE_REF_CNT {
            self.incr_glyphs.push(self.lifetime);
        }
        abs_ref
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
    pub fn increment_lifetime(&mut self) {
        self.lifetime += 2;
    }

    pub fn gc(&mut self, threshold: u64) -> Vec<Fingerprint> {
        let mut gc_items = vec![];

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

    pub fn finalize_delta(&mut self) -> Module {
        let glyphs = GlyphPackBuilder::finalize(
            self.glyphs
                .iter()
                .filter(|e| self.incr_glyphs[e.1.id.0 as usize] == self.lifetime)
                .map(|(x, y)| (x.clone(), y.clone())),
        );

        let items = ItemMap::from_iter(
            self.items
                .iter()
                .filter(|(_, e)| e.0 == self.lifetime)
                .map(|(f, (_, i))| (*f, i.clone())),
        );

        Module {
            glyphs,
            items,
            source_mapping: self.source_mapping.clone(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ModuleMetadata {
    SourceMappingData(Vec<SourceMappingNode>),
    PageSourceMapping(Vec<Vec<SourceMappingNode>>),
    GarbageCollection(Vec<Fingerprint>),
}

/// Flatten transform item.
#[derive(Debug)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct SerializedModule {
    pub metadata: Vec<ModuleMetadata>,
    pub item_pack: ItemPack,
    pub glyphs: Vec<(AbsoluteRef, FlatGlyphItem)>,
    pub layouts: Vec<(Abs, Vec<(Fingerprint, Size)>)>,
}

// todo: remove me
pub fn serialize_module(repr: Module) -> Vec<u8> {
    // Or you can customize your serialization for better performance
    // and compatibility with #![no_std] environments
    use rkyv::ser::{serializers::AllocSerializer, Serializer};

    let mut serializer = AllocSerializer::<0>::default();
    serializer
        .serialize_value(&ItemPack(repr.items.into_iter().collect()))
        .unwrap();
    let item_pack = serializer.into_serializer().into_inner();

    item_pack.into_vec()
}

pub fn build_flat_glyphs(
    repr: impl IntoIterator<Item = (GlyphItem, AbsoluteRef)>,
) -> Vec<(AbsoluteRef, FlatGlyphItem)> {
    let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
    let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider);

    repr.into_iter()
        .flat_map(|(glyph, glyph_id)| {
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

pub fn serialize_module_v2(repr: &SerializedModule) -> Vec<u8> {
    // Or you can customize your serialization for better performance
    // and compatibility with #![no_std] environments
    use rkyv::ser::{serializers::AllocSerializer, Serializer};

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(repr).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    bytes.into_vec()
}

pub fn serialize_multi_doc_standalone(
    doc: MultiSvgDocument,
    glyph_mapping: GlyphMapping,
) -> Vec<u8> {
    let glyphs = build_flat_glyphs(glyph_mapping);

    serialize_module_v2(&SerializedModule {
        metadata: vec![],
        item_pack: ItemPack(doc.module.items.into_iter().collect()),
        glyphs,
        layouts: doc.layouts,
    })
}
