use std::cell::OnceCell;

use reflexo::{
    hash::Fingerprint,
    vector::{
        incr::IncrDocClient,
        ir::{Module, Page},
    },
};
use web_sys::wasm_bindgen::JsCast;

use crate::{BrowserFontMetric, SemaTask};

#[derive(Clone)]
pub struct SemaPage {
    content: Fingerprint,
    cache: Option<(String, bool)>,
}

/// Incremental pass from vector to Sema
#[derive(Default)]
pub struct IncrVec2SemaPass {
    pages: Vec<SemaPage>,
    metric: OnceCell<BrowserFontMetric>,
}

impl IncrVec2SemaPass {
    /// Interprets the changes in the given module and pages.
    pub fn interpret_changes(&mut self, module: &Module, pages: &[Page]) {
        let pages = pages
            .iter()
            .enumerate()
            .map(|(idx, Page { content, size })| {
                if idx < self.pages.len() && self.pages[idx].content == *content {
                    return self.pages[idx].clone();
                }

                let metric = self.metric.get_or_init(|| {
                    let canvas = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .create_element("canvas")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlCanvasElement>()
                        .unwrap();
                    BrowserFontMetric::new(&canvas)
                });

                let mut t = SemaTask::new(true, *metric, size.x.0, size.y.0);
                let mut output = vec![];
                let ts = tiny_skia::Transform::identity();
                t.render_semantics(module, ts, *content, &mut output);

                SemaPage {
                    content: *content,
                    cache: Some((output.concat(), true)),
                    // size: *size,
                }
            })
            .collect();
        self.pages = pages;
    }
}

/// Maintains the state of the incremental rendering HTML semantics at client
/// side
#[derive(Default)]
pub struct IncrSemaDocClient {
    /// State of converting vector to Sema
    pub vec2sema: IncrVec2SemaPass,
}

impl IncrSemaDocClient {
    /// Reset the state of the incremental rendering.
    pub fn reset(&mut self) {
        self.vec2sema.pages.clear();
    }

    pub fn patch_delta(&mut self, kern: &IncrDocClient) {
        if let Some(layout) = &kern.layout {
            let pages = layout.pages(&kern.doc.module);
            if let Some(pages) = pages {
                self.vec2sema
                    .interpret_changes(pages.module(), pages.pages());
            }
        }
    }

    pub fn page(&mut self, idx: usize, heavy: bool) -> Option<String> {
        let _ = heavy;
        Some(self.vec2sema.pages.get(idx)?.cache.as_ref()?.0.clone())
    }
}
