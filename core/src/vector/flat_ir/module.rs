use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};

use comemo::Prehashed;

use crate::{
    hash::Fingerprint,
    vector::ir::{AbsoluteRef, DefId, GlyphItem},
};

use super::{FlatGlyphItem, FlatSvgItem, ItemPack, LayoutRegion, SourceMappingNode};

pub type ItemMap = BTreeMap<Fingerprint, FlatSvgItem>;

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
    fn layouts(&self) -> Arc<LayoutRegion>;
    fn glyphs(&self) -> Vec<(DefId, FlatGlyphItem)>;
    fn gc_items(&self) -> Option<Vec<Fingerprint>> {
        // never gc items
        None
    }
}

/// A finished module that stores all the svg items.
/// The svg items shares the underlying data.
/// The svg items are flattened and ready to be serialized.
#[derive(Debug, Default, Clone, Hash)]
pub struct Module {
    pub glyphs: Vec<(DefId, GlyphItem)>,
    pub items: ItemMap,
    pub source_mapping: Vec<SourceMappingNode>,
}

impl Module {
    pub fn freeze(self) -> FrozenModule {
        FrozenModule(Arc::new(Prehashed::new(self)))
    }

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
