use std::sync::Arc;

use typst::model::Document;

use super::ir::{
    FlatGlyphItem, FlatModule, GlyphRef, IncrFontPack, IncrGlyphPack, ItemPack, LayoutRegion,
    LayoutRegionNode, LayoutSourceMapping, Module, ModuleMetadata, MultiVecDocument, Page,
    SourceMappingNode, VecDocument,
};
use super::pass::IncrTypst2VecPass;
use crate::debug_loc::ElementPoint;
use crate::{debug_loc::SourceSpanOffset, error::prelude::*, TakeAs};

/// maintains the data of the incremental rendering at server side
#[derive(Default)]
pub struct IncrDocServer {
    /// Expected exact state of the current Compiler.
    /// Initially it is None meaning no completed compilation.
    doc_view: Option<VecDocument>,

    /// Maintaining typst -> vector status
    typst2vec: IncrTypst2VecPass,
}

impl IncrDocServer {
    pub fn set_should_attach_debug_info(&mut self, should_attach_debug_info: bool) {
        self.typst2vec
            .spans
            .set_should_attach_debug_info(should_attach_debug_info);
    }

    /// Pack the delta into a binary blob.
    pub fn pack_delta(&mut self, output: Arc<Document>) -> Vec<u8> {
        self.typst2vec.spans.reset();

        // Increment the lifetime of all items to touch.
        self.typst2vec.increment_lifetime();

        // it is important to call gc before building pages
        let gc_items = self.typst2vec.gc(5 * 2);

        // run typst2vec pass
        let pages = self.typst2vec.doc(&output.introspector, &output);

        // let new_items = builder.new_items.get_mut().len();
        // let new_fonts = builder.glyphs.new_fonts.get_mut().len();
        // let new_glyphs = builder.glyphs.new_glyphs.get_mut().len();

        let delta = self.typst2vec.finalize_delta();

        // max, min lifetime current, gc_items
        #[cfg(feature = "debug-gc")]
        {
            let mi = self
                .module_builder
                .items
                .clone()
                .into_iter()
                .map(|i| i.1 .0)
                .min()
                .unwrap_or(0);
            println!(
                "gc[{}]: i/f/g: {}/{}/{} max: {}, min: {}, remove: {}",
                self.module_builder.lifetime,
                new_items,
                new_fonts,
                new_glyphs,
                self.module_builder
                    .items
                    .clone()
                    .into_iter()
                    .map(|i| i.1 .0)
                    .max()
                    .unwrap_or(0xffffffff),
                mi,
                gc_items.len()
            );

            // for (fg, (_, item)) in
            //     self.module_builder.items.iter().filter(|(_, i)| i.0 == mi) {
            //     println!("mi {fg:?} => {item:#?}");
            // }
        }

        let fonts = IncrFontPack {
            items: delta.fonts,
            incremental_base: 0, // todo: correct incremental_base
        };

        let glyphs = IncrGlyphPack {
            items: delta.glyphs,
            incremental_base: 0, // todo: correct incremental_base
        };

        let pages = LayoutRegionNode::new_pages(pages.clone());
        let pages = Arc::new(vec![LayoutRegion::new_single(pages)]);

        let delta = FlatModule::new(vec![
            ModuleMetadata::GarbageCollection(gc_items),
            ModuleMetadata::Font(Arc::new(fonts)),
            ModuleMetadata::Glyph(Arc::new(glyphs)),
            ModuleMetadata::Item(ItemPack(delta.items.clone().into_iter().collect())),
            ModuleMetadata::Layout(pages),
        ])
        .to_bytes();

        // log::info!("svg render time (incremental bin): {:?}", instant.elapsed());
        [b"diff-v1,", delta.as_slice()].concat()
    }

    /// Pack the current entirely into a binary blob.
    pub fn pack_current(&mut self) -> Option<Vec<u8>> {
        let doc = self.doc_view.as_ref()?;

        let (fonts, glyphs) = self.typst2vec.glyphs.finalize();

        let pages = LayoutRegionNode::new_pages(doc.pages.clone());
        let pages = Arc::new(vec![LayoutRegion::new_single(pages)]);

        let delta = FlatModule::new(vec![
            // todo: correct incremental_base
            ModuleMetadata::Font(Arc::new(fonts.into())),
            ModuleMetadata::Glyph(Arc::new(glyphs.into())),
            ModuleMetadata::Item(ItemPack(doc.module.items.clone().into_iter().collect())),
            ModuleMetadata::Layout(pages),
        ])
        .to_bytes();
        Some([b"new,", delta.as_slice()].concat())
    }

    /// Gets element paths by the given span.
    ///
    /// See [`crate::vector::pass::Span2VecPass::query_element_paths`] for more
    /// information.
    pub fn resolve_element_paths_by_span(
        &mut self,
        span_offset: SourceSpanOffset,
    ) -> ZResult<Vec<Vec<ElementPoint>>> {
        self.typst2vec.spans.query_element_paths(span_offset)
    }

    /// Gets the span range of the given element path.
    pub fn resolve_span_by_element_path(
        &mut self,
        path: &[ElementPoint],
    ) -> ZResult<Option<(SourceSpanOffset, SourceSpanOffset)>> {
        self.typst2vec.spans.query(path)
    }
}

/// maintains the data of the incremental rendering at client side
#[derive(Default)]
pub struct IncrDocClient {
    /// Full information of the current document from server.
    pub doc: MultiVecDocument,
    /// Hold glyphs.
    pub glyphs: Vec<(GlyphRef, FlatGlyphItem)>,

    /// checkout of the current document.
    pub layout: Option<LayoutRegionNode>,
    /// Optional source mapping data.
    pub source_mapping_data: Vec<SourceMappingNode>,
    /// Optional page source mapping references.
    pub page_source_mappping: LayoutSourceMapping,
}

impl IncrDocClient {
    /// Merge the delta from server.
    pub fn merge_delta(&mut self, delta: FlatModule) {
        self.doc.merge_delta(&delta);
        for metadata in delta.metadata {
            match metadata {
                ModuleMetadata::Glyph(data) => {
                    self.glyphs.extend(data.take().items.into_iter());
                }
                ModuleMetadata::SourceMappingData(data) => {
                    self.source_mapping_data = data;
                }
                ModuleMetadata::PageSourceMapping(data) => {
                    self.page_source_mappping = data.take();
                }
                _ => {}
            }
        }
    }

    /// Set the current layout of the document.
    /// This is so bare-bone that stupidly takes a selected layout.
    ///
    /// Please wrap this for your own use case.
    pub fn set_layout(&mut self, layout: LayoutRegionNode) {
        self.layout = Some(layout);
    }

    /// Kern of the client without leaking abstraction.
    pub fn kern(&self) -> IncrDocClientKern<'_> {
        IncrDocClientKern::new(self)
    }

    pub fn module(&self) -> &Module {
        &self.doc.module
    }

    pub fn module_mut(&mut self) -> &mut Module {
        &mut self.doc.module
    }
}

fn access_slice<'a, T>(v: &'a [T], idx: usize, kind: &'static str, pos: usize) -> ZResult<&'a T> {
    v.get(idx).ok_or_else(
        || error_once!("out of bound access", pos: pos, kind: kind, idx: idx, actual: v.len()),
    )
}

pub struct IncrDocClientKern<'a>(&'a IncrDocClient);

impl<'a> IncrDocClientKern<'a> {
    pub fn new(client: &'a IncrDocClient) -> Self {
        Self(client)
    }

    /// Get current pages meta of the selected document.
    pub fn pages_meta(&self) -> Option<&[Page]> {
        let layout = self.0.layout.as_ref();
        layout.and_then(LayoutRegionNode::pages_meta)
    }

    /// Get estimated width of the document (in flavor of PDF Viewer).
    pub fn doc_width(&self) -> Option<f32> {
        let view = self.pages_meta()?.iter();
        Some(view.map(|p| p.size.x).max().unwrap_or_default().0)
    }

    /// Get estimated height of the document (in flavor of PDF Viewer).
    pub fn doc_height(&self) -> Option<f32> {
        let view = self.pages_meta()?.iter();
        Some(view.map(|p| p.size.y.0).sum())
    }

    /// Get the source location of the given path.
    pub fn source_span(&self, path: &[u32]) -> ZResult<Option<String>> {
        const SOURCE_MAPPING_TYPE_TEXT: u32 = 0;
        const SOURCE_MAPPING_TYPE_GROUP: u32 = 1;
        const SOURCE_MAPPING_TYPE_IMAGE: u32 = 2;
        const SOURCE_MAPPING_TYPE_SHAPE: u32 = 3;
        const SOURCE_MAPPING_TYPE_PAGE: u32 = 4;

        if self.0.page_source_mappping.is_empty() {
            return Ok(None);
        }

        let mut index_item: Option<&SourceMappingNode> = None;

        let source_mapping = self.0.source_mapping_data.as_slice();
        let page_sources = self.0.page_source_mappping[0]
            .source_mapping(&self.0.doc.module)
            .unwrap();
        let page_sources = page_sources.source_mapping();

        for (chunk_idx, v) in path.chunks_exact(2).enumerate() {
            let (ty, idx) = (v[0], v[1] as usize);

            let this_item: &SourceMappingNode = match index_item {
                Some(SourceMappingNode::Group(q)) => {
                    let idx = *access_slice(q, idx, "group_index", chunk_idx)? as usize;
                    access_slice(source_mapping, idx, "source_mapping", chunk_idx)?
                }
                Some(_) => {
                    return Err(error_once!("cannot index", pos:
        chunk_idx, indexing: format!("{:?}", index_item)))
                }
                None => access_slice(page_sources, idx, "page_sources", chunk_idx)?,
            };

            match (ty, this_item) {
                (SOURCE_MAPPING_TYPE_PAGE, SourceMappingNode::Page(page_index)) => {
                    index_item = Some(access_slice(
                        source_mapping,
                        *page_index as usize,
                        "source_mapping",
                        chunk_idx,
                    )?);
                }
                (SOURCE_MAPPING_TYPE_GROUP, SourceMappingNode::Group(_)) => {
                    index_item = Some(this_item);
                }
                (SOURCE_MAPPING_TYPE_TEXT, SourceMappingNode::Text(n))
                | (SOURCE_MAPPING_TYPE_IMAGE, SourceMappingNode::Image(n))
                | (SOURCE_MAPPING_TYPE_SHAPE, SourceMappingNode::Shape(n)) => {
                    return Ok(Some(format!("{n:x}")));
                }
                _ => {
                    return Err(error_once!("invalid/mismatch node
                    type",                         pos: chunk_idx, ty: ty,
                        actual: format!("{:?}", this_item),
                        parent: format!("{:?}", index_item),
                        child_idx_in_parent: idx,
                    ))
                }
            }
        }

        Ok(None)
    }
}
