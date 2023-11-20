use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::Arc,
};

use comemo::Prehashed;
use typst::font::Font;

use crate::{
    hash::{Fingerprint, FingerprintBuilder},
    vector::ir::{
        AbsoluteRef, BuildGlyph, DefId, FontItem, FontRef, GlyphItem, GlyphPackBuilderImpl,
        GlyphRef, SvgItem,
    },
    TakeAs,
};

use super::{
    FlatSvgItem, FlatTextItem, FlatTextItemContent, FontPack, GlyphPack, GroupRef, ItemPack,
    LayoutRegion, SourceMappingNode, TransformedRef,
};

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
    fn layouts(&self) -> Arc<Vec<LayoutRegion>>;
    fn fonts(&self) -> Arc<FontPack>;
    fn glyphs(&self) -> Arc<GlyphPack>;
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
    pub fonts: Vec<FontItem>,
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

/// Intermediate representation of a incompleted svg item.
pub struct ModuleBuilderImpl<const ENABLE_REF_CNT: bool = false> {
    pub glyphs: GlyphPackBuilderImpl<ENABLE_REF_CNT>,
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
    fn build_font(&mut self, font: &Font) -> FontRef {
        self.glyphs.build_font(font)
    }

    fn build_glyph(&mut self, glyph: &GlyphItem) -> GlyphRef {
        self.glyphs.build_glyph(glyph)
    }
}

impl<const ENABLE_REF_CNT: bool> ModuleBuilderImpl<ENABLE_REF_CNT> {
    pub fn reset(&mut self) {
        self.source_mapping.clear();
        self.source_mapping_buffer.clear();
    }

    pub fn finalize_ref(&self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
            items: self.items.clone().to_item_map(),
            source_mapping: self.source_mapping.clone(),
        }
    }

    pub fn finalize(self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
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
            SvgItem::Gradient(g) => FlatSvgItem::Gradient(g),
            SvgItem::Link(link) => FlatSvgItem::Link(link),
            SvgItem::Text(text) => {
                let font = self.build_font(&text.font);
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
                    font,
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
            SvgItem::Group(group, size) => {
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
                    // Safety: `t` is `Some` only if `should_attach_debug_info` is `true`.
                    let sm_start = unsafe { t.unwrap_unchecked() };
                    let sm_range = self
                        .source_mapping_buffer
                        .splice(sm_start..self.source_mapping_buffer.len(), []);

                    let sm_id = self.source_mapping.len() as u64;
                    self.source_mapping
                        .push(SourceMappingNode::Group(sm_range.collect()));
                    self.source_mapping_buffer.push(sm_id);
                }

                FlatSvgItem::Group(GroupRef(items.into()), size)
            }
            SvgItem::ContentHint(c) => FlatSvgItem::ContentHint(c),
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
        self.glyphs.lifetime = self.lifetime;
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
        // filter glyphs by lifetime
        let (fonts, glyphs) = self.glyphs.finalize_delta();

        // filter items by lifetime
        let items = {
            let items = self.items.iter();
            let items = items.filter(|(_, e)| e.0 == self.lifetime);
            let items = items.map(|(f, (_, i))| (*f, i.clone()));

            ItemMap::from_iter(items)
        };

        Module {
            fonts,
            glyphs,
            items,
            source_mapping: self.source_mapping.clone(),
        }
    }
}
