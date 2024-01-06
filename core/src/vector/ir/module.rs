use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

use comemo::Prehashed;

use crate::{hash::Fingerprint, ImmutStr, TakeAs};

use super::{preludes::*, *};

pub type ItemMap = BTreeMap<Fingerprint, VecItem>;

pub type RefItemMap = HashMap<Fingerprint, (u64, VecItem)>;
pub type RefItemMapSync = chashmap::CHashMap<Fingerprint, (u64, VecItem)>;

pub trait ToItemMap {
    fn to_item_map(self) -> ItemMap;
}

impl ToItemMap for RefItemMap {
    fn to_item_map(self) -> ItemMap {
        self.into_iter().map(|(k, (_, v))| (k, v)).collect::<_>()
    }
}

impl ToItemMap for RefItemMapSync {
    fn to_item_map(self) -> ItemMap {
        self.into_iter().map(|(k, (_, v))| (k, v)).collect::<_>()
    }
}

/// Trait of a streaming representation of a module.
pub trait ModuleStream {
    fn items(&self) -> ItemPack;
    fn layouts(&self) -> Arc<Vec<LayoutRegion>>;
    fn fonts(&self) -> Arc<IncrFontPack>;
    fn glyphs(&self) -> Arc<IncrGlyphPack>;
    fn gc_items(&self) -> Option<Vec<Fingerprint>> {
        // never gc items
        None
    }
}

/// A finished module that stores all the vector items.
/// The vector items shares the underlying data.
/// The vector items are flattened and ready to be serialized.
#[derive(Debug, Default, Clone, Hash)]
pub struct Module {
    pub fonts: Vec<FontItem>,
    pub glyphs: Vec<(DefId, GlyphItem)>,
    pub items: ItemMap,
}

impl Module {
    pub fn freeze(self) -> FrozenModule {
        FrozenModule(Arc::new(Prehashed::new(self)))
    }

    /// Get a font item by its stable ref.
    pub fn get_font(&self, id: &FontRef) -> Option<&FontItem> {
        self.fonts.get(id.idx as usize)
    }

    /// Get a glyph item by its stable ref.
    pub fn get_glyph(&self, id: &AbsoluteRef) -> Option<&GlyphItem> {
        self.glyphs.get(id.id.0 as usize).map(|(_, item)| item)
    }

    /// Get a svg item by its stable ref.
    pub fn get_item(&self, id: &Fingerprint) -> Option<&VecItem> {
        self.items.get(id)
    }

    pub fn merge_delta(&mut self, v: impl ModuleStream) {
        let item_pack: ItemPack = v.items();
        if let Some(gc_items) = v.gc_items() {
            for id in gc_items {
                self.items.remove(&id);
            }
        }
        self.items.extend(item_pack.0);

        let fonts = v.fonts();
        self.fonts.extend(fonts.take().items);

        let glyphs = v.glyphs();
        self.glyphs.extend(
            glyphs
                .take()
                .items
                .into_iter()
                .map(|(id, item)| (id, item.into())),
        );
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FrozenModule(pub Arc<Prehashed<Module>>);

pub struct ModuleView {
    module: Module,
}

impl ModuleView {
    /// See [`std::path::Path`]
    pub fn new<M: AsRef<Module> + ?Sized>(m: &M) -> &Self {
        unsafe { &*(m.as_ref() as *const Module as *const ModuleView) }
    }
}

impl ToOwned for ModuleView {
    type Owned = Module;

    fn to_owned(&self) -> Self::Owned {
        self.module.clone()
    }
}

impl AsRef<Module> for ModuleView {
    #[inline]
    fn as_ref(&self) -> &Module {
        &self.module
    }
}

impl AsRef<Module> for Module {
    #[inline]
    fn as_ref(&self) -> &Module {
        self
    }
}

impl AsRef<Module> for FrozenModule {
    #[inline]
    fn as_ref(&self) -> &Module {
        self.0.deref().deref()
    }
}

impl AsRef<FrozenModule> for FrozenModule {
    #[inline]
    fn as_ref(&self) -> &FrozenModule {
        self
    }
}

impl Borrow<ModuleView> for FrozenModule {
    fn borrow(&self) -> &ModuleView {
        ModuleView::new(self)
    }
}

impl Borrow<ModuleView> for Module {
    fn borrow(&self) -> &ModuleView {
        ModuleView::new(self)
    }
}

impl Borrow<Module> for FrozenModule {
    fn borrow(&self) -> &Module {
        self.0.deref().deref()
    }
}

/// metadata that can be attached to a module.
#[derive(Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
#[repr(C, align(32))]
pub enum PageMetadata {
    GarbageCollection(Vec<Fingerprint>),
    Item(ItemPack),
    Glyph(Arc<IncrGlyphPack>),
    Custom(Vec<(ImmutStr, ImmutBytes)>),
}

impl fmt::Debug for PageMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageMetadata::GarbageCollection(v) => f
                .debug_struct("GarbageCollection")
                .field("len", &v.len())
                .finish(),
            PageMetadata::Item(v) => f.debug_struct("Item").field("len", &v.0.len()).finish(),
            PageMetadata::Glyph(v) => f
                .debug_struct("Glyph")
                .field("len", &v.items.len())
                .field("base", &v.incremental_base)
                .finish(),
            PageMetadata::Custom(v) => {
                write!(f, "Custom")?;
                f.debug_map()
                    .entries(
                        v.iter()
                            .map(|(k, v)| (k.as_ref(), format!("Bytes({})", v.len()))),
                    )
                    .finish()
            }
        }
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
    Font(Arc<IncrFontPack>),
    Glyph(Arc<IncrGlyphPack>),
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

    fn fonts(&self) -> Arc<IncrFontPack> {
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

    fn glyphs(&self) -> Arc<IncrGlyphPack> {
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
