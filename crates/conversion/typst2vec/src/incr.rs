use reflexo::error::prelude::*;
use reflexo::typst::TypstDocument;
use reflexo::vector::ir::{ModuleMetadata, Page};

use super::ir::FlatModule;
use super::pass::IncrTypst2VecPass;
use crate::debug_loc::{ElementPoint, SourceSpanOffset};

/// Client side implementation is free from typst details.
pub use reflexo::vector::incr::{IncrDocClient, IncrDocClientKern};

/// maintains the data of the incremental rendering at server side
#[derive(Default)]
pub struct IncrDocServer {
    /// The references of pages of the document.
    /// Initially it is None meaning no completed compilation.
    pages: Option<Vec<Page>>,

    /// Maintaining typst -> vector status
    typst2vec: IncrTypst2VecPass,
}

impl IncrDocServer {
    /// Set whether to attach debug info to the spans.
    pub fn set_should_attach_debug_info(&mut self, should_attach_debug_info: bool) {
        self.typst2vec
            .spans
            .set_should_attach_debug_info(should_attach_debug_info);
    }

    /// Pack the delta into a binary blob.
    pub fn pack_delta(&mut self, output: &TypstDocument) -> Vec<u8> {
        self.typst2vec.spans.reset();

        // Increment the lifetime of all items to touch.
        self.typst2vec.increment_lifetime();

        // it is important to call gc before building pages
        let gc_items = self.typst2vec.gc(5 * 2);

        // run typst2vec pass
        let pages = self.typst2vec.doc(output);
        self.pages = Some(pages.clone());

        // let new_items = builder.new_items.get_mut().len();
        // let new_fonts = builder.glyphs.new_fonts.get_mut().len();
        // let new_glyphs = builder.glyphs.new_glyphs.get_mut().len();

        let delta = self.typst2vec.finalize_delta();

        // max, min lifetime current, gc_items
        #[cfg(feature = "debug-gc")]
        {
            let mi = self
                .typst2vec
                .items
                .clone()
                .into_iter()
                .map(|i| i.1 .0)
                .min()
                .unwrap_or(0);
            eprintln!(
                "gc[{}]: max: {}, min: {}, remove: {}",
                self.typst2vec.lifetime,
                self.typst2vec
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
            //     self.typst2vec.items.iter().filter(|(_, i)| i.0 == mi) {
            //     eprintln!("mi {fg:?} => {item:#?}");
            // }
        }

        let mut m = FlatModule::with_capacity(5);
        m.push(ModuleMetadata::GarbageCollection(gc_items));
        m.add_module(delta);
        m.add_single_layout(pages);
        let delta = m.to_bytes();

        // log::info!("svg render time (incremental bin): {:?}", instant.elapsed());
        [b"diff-v1,", delta.as_slice()].concat()
    }

    /// Pack the current entirely into a binary blob.
    pub fn pack_current(&mut self) -> Option<Vec<u8>> {
        let pages = self.pages.as_ref()?.clone();
        let full = self.typst2vec.finalize_ref();

        let mut m = FlatModule::with_capacity(4);
        m.add_module(full);
        m.add_single_layout(pages);
        let full = m.to_bytes();

        Some([b"new,", full.as_slice()].concat())
    }

    /// Gets element paths by the given span.
    ///
    /// See [`crate::pass::Span2VecPass::query_element_paths`] for more
    /// information.
    pub fn resolve_element_paths_by_span(
        &mut self,
        span_offset: SourceSpanOffset,
    ) -> Result<Vec<Vec<ElementPoint>>> {
        self.typst2vec.spans.query_element_paths(span_offset)
    }

    /// Gets the span range of the given element path.
    pub fn resolve_span_by_element_path(
        &mut self,
        path: &[ElementPoint],
    ) -> Result<Option<(SourceSpanOffset, SourceSpanOffset)>> {
        self.typst2vec.spans.query(path)
    }
}
