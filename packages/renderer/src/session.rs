use std::sync::{Arc, Mutex};

#[cfg(feature = "render_canvas")]
use typst_ts_canvas_exporter::IncrCanvasDocClient;
use typst_ts_core::{
    error::prelude::*,
    vector::{flat_ir::Page, incr::IncrDocClient, ir::Scalar},
};
#[cfg(feature = "render_svg")]
use typst_ts_svg_exporter::IncrSvgDocClient;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Default, Debug)]
pub struct RenderSessionOptions {
    pub(crate) pixel_per_pt: Option<f32>,
    pub(crate) background_color: Option<String>,
    pub(crate) format: Option<String>,
}

#[wasm_bindgen]
impl RenderSessionOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenderSessionOptions {
        Self {
            pixel_per_pt: None,
            background_color: None,
            format: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> Option<f32> {
        self.pixel_per_pt
    }

    #[wasm_bindgen(setter)]
    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        self.pixel_per_pt = Some(pixel_per_pt);
    }

    #[wasm_bindgen(getter)]
    pub fn background_color(&self) -> Option<String> {
        self.background_color.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_background_color(&mut self, background_color: String) {
        self.background_color = Some(background_color);
    }

    #[wasm_bindgen(getter)]
    pub fn format(&self) -> Option<String> {
        self.format.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_format(&mut self, format: String) {
        self.format = Some(format);
    }
}

#[wasm_bindgen]
#[derive(Default, Debug)]
pub struct CreateSessionOptions {
    pub(crate) format: Option<String>,
    pub(crate) artifact_content: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl CreateSessionOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            format: None,
            artifact_content: None,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_format(&mut self, format: String) {
        self.format = Some(format);
    }

    #[wasm_bindgen(setter)]
    pub fn set_artifact_content(&mut self, artifact_content: Vec<u8>) {
        self.artifact_content = Some(artifact_content);
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PageInfo {
    pub(crate) page_off: usize,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

#[wasm_bindgen]
impl PageInfo {
    #[wasm_bindgen(getter)]
    pub fn page_off(&self) -> usize {
        self.page_off
    }

    #[wasm_bindgen(getter)]
    pub fn width_pt(&self) -> f64 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height_pt(&self) -> f64 {
        self.height
    }
}

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct PagesInfo {
    pub(crate) pages: Vec<PageInfo>,
}

#[wasm_bindgen]
impl PagesInfo {
    #[wasm_bindgen(getter)]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn page_by_number(&self, num: usize) -> Option<PageInfo> {
        for page in &self.pages {
            if page.page_off == num {
                return Some(page.clone());
            }
        }
        None
    }

    pub fn page(&self, i: usize) -> PageInfo {
        self.pages[i].clone()
    }

    pub fn width(&self) -> f32 {
        self.pages
            .iter()
            .map(|s| Scalar(s.width as f32))
            .max()
            .unwrap_or_default()
            .0
    }

    pub fn height(&self) -> f32 {
        self.pages.iter().map(|s| s.height as f32).sum()
    }
}

#[derive(Default)]
#[wasm_bindgen]
pub struct RenderSession {
    /// pixel per point
    /// Only used for canvas rendering
    pub(crate) pixel_per_pt: Option<f32>,

    /// background color
    /// Only used for canvas rendering
    pub(crate) background_color: Option<String>,

    /// stored pages info
    pub(crate) pages_info: PagesInfo,

    /// underlying communication client model
    pub(crate) client: Arc<Mutex<IncrDocClient>>,
    /// underlying incremental state of canvas rendering
    #[cfg(feature = "render_canvas")]
    pub(crate) canvas_kern: Arc<Mutex<IncrCanvasDocClient>>,
    /// underlying incremental state of svg rendering
    #[cfg(feature = "render_svg")]
    pub(crate) svg_kern: Arc<Mutex<IncrSvgDocClient>>,
}

#[wasm_bindgen]
impl RenderSession {
    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> Option<f32> {
        self.pixel_per_pt
    }

    #[wasm_bindgen(setter)]
    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        self.pixel_per_pt = Some(pixel_per_pt);
    }

    #[wasm_bindgen(getter)]
    pub fn background_color(&self) -> Option<String> {
        self.background_color.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_background_color(&mut self, background_color: String) {
        self.background_color = Some(background_color);
    }

    #[wasm_bindgen(getter)]
    pub fn pages_info(&self) -> PagesInfo {
        self.pages_info.clone()
    }
}

#[wasm_bindgen]
impl RenderSession {
    pub(crate) fn client(&self) -> std::sync::MutexGuard<'_, IncrDocClient> {
        self.client.lock().unwrap()
    }

    #[wasm_bindgen(getter)]
    pub fn doc_width(&self) -> f32 {
        self.client().kern().doc_width().unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn doc_height(&self) -> f32 {
        self.client().kern().doc_height().unwrap_or_default()
    }

    pub fn source_span(&self, path: &[u32]) -> ZResult<Option<String>> {
        self.client().kern().source_span(path)
    }

    pub(crate) fn reset(&mut self) {
        let mut client = self.client.lock().unwrap();
        *client = IncrDocClient::default();
        if cfg!(feature = "render_canvas") {
            let mut canvas_kern = self.canvas_kern.lock().unwrap();
            canvas_kern.reset();
        }
        if cfg!(feature = "render_svg") {
            let mut svg_kern = self.svg_kern.lock().unwrap();
            svg_kern.reset();
        }
    }

    pub(crate) fn reset_current(&mut self, delta: &[u8]) -> ZResult<()> {
        let mut client = self.client.lock().unwrap();
        *client = IncrDocClient::default();
        if cfg!(feature = "render_canvas") {
            let mut canvas_kern = self.canvas_kern.lock().unwrap();
            canvas_kern.reset();
        }
        if cfg!(feature = "render_svg") {
            let mut svg_kern = self.svg_kern.lock().unwrap();
            svg_kern.reset();
        }
        Self::merge_delta_inner(&mut self.pages_info, &mut client, delta)
    }

    pub(crate) fn merge_delta(&mut self, delta: &[u8]) -> ZResult<()> {
        let mut client = self.client.lock().unwrap();
        Self::merge_delta_inner(&mut self.pages_info, &mut client, delta)
    }

    pub(crate) fn merge_delta_inner(
        pages_info: &mut PagesInfo,
        client: &mut IncrDocClient,
        delta: &[u8],
    ) -> ZResult<()> {
        use typst_ts_core::vector::stream::BytesModuleStream;

        let delta = BytesModuleStream::from_slice(delta).checkout_owned();
        let _delta_ref = &delta;

        #[cfg(feature = "debug_delta_update")]
        use typst_ts_core::vector::flat_ir::ModuleStream;

        #[cfg(feature = "debug_delta_update")]
        crate::utils::console_log!(
            "module counts: g:{:?},i:{:?},l:{:?},gc:{:?}",
            _delta_ref.glyphs().items.len(),
            _delta_ref.items().0.len(),
            _delta_ref.layouts().len(),
            _delta_ref.gc_items().map(|s| s.len()),
        );

        client.merge_delta(delta);
        // checkout the current layout
        // todo: multiple layout
        let layouts = &client.doc.layouts[0];
        if !layouts.is_empty() {
            let layout = layouts.unwrap_single();
            client.set_layout(layout);
        }

        // checkout the current pages
        let pages = if let Some(layout) = &client.layout {
            let mut pages = vec![];
            let view = layout.pages(&client.doc.module);
            if let Some(view) = view {
                // Vec::with_capacity(client.elements.pages.len());
                pages.reserve(view.pages().len());
                for (i, Page { size, .. }) in view.pages().iter().enumerate() {
                    pages.push(PageInfo {
                        page_off: i,
                        width: size.x.0 as f64,
                        height: size.y.0 as f64,
                    });
                }
            }

            pages
        } else {
            vec![]
        };

        *pages_info = PagesInfo { pages };
        Ok(())
    }
}
