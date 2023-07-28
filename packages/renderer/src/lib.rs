#[macro_use]
pub(crate) mod utils;

use typst_ts_core::error::prelude::*;
use typst_ts_core::font::FontResolverImpl;
use typst_ts_core::vector::geom::Axes;
use typst_ts_core::vector::geom::Scalar;
use typst_ts_svg_exporter::flat_ir::SourceMappingNode;
use typst_ts_svg_exporter::{
    DefaultExportFeature, IncrementalSvgV2Exporter, LayoutElem, Pages, SvgExporter,
};
use wasm_bindgen::prelude::*;

pub(crate) mod parser;

pub(crate) mod builder;
pub use builder::TypstRendererBuilder;

pub(crate) mod render;

pub(crate) mod session;
pub use session::RenderSession;

pub use session::{RenderSessionManager, RenderSessionOptions};

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct RenderPageImageOptions {
    pub(crate) page_off: usize,
}

#[wasm_bindgen]
impl RenderPageImageOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { page_off: 0 }
    }

    #[wasm_bindgen(getter)]
    pub fn page_off(&self) -> usize {
        self.page_off
    }

    #[wasm_bindgen(setter)]
    pub fn set_page_off(&mut self, page_off: usize) {
        self.page_off = page_off;
    }
}

#[wasm_bindgen]
pub struct TypstRenderer {
    pub(crate) session_mgr: RenderSessionManager,
}

#[wasm_bindgen]
pub struct SvgSession {
    doc: typst_ts_svg_exporter::MultiSvgDocument,
    doc_view: Option<Pages>,
    source_mapping_data: Vec<SourceMappingNode>,
    page_source_mappping: Vec<Vec<SourceMappingNode>>,
}

#[wasm_bindgen]
impl SvgSession {
    pub fn reset(&mut self) {
        self.doc = Default::default();
        self.doc_view = None;
        self.source_mapping_data = Default::default();
        self.page_source_mappping = Default::default();
    }

    pub fn merge_delta(&mut self, delta: &[u8]) -> ZResult<()> {
        let delta = typst_ts_svg_exporter::flat_ir::stream::SvgDocumentStream::from_slice(delta);
        let delta = delta.checkout_owned();

        self.doc.merge_delta(&delta);
        for metadata in delta.metadata {
            match metadata {
                typst_ts_svg_exporter::flat_ir::ModuleMetadata::SourceMappingData(data) => {
                    self.source_mapping_data = data;
                }
                typst_ts_svg_exporter::flat_ir::ModuleMetadata::PageSourceMapping(data) => {
                    self.page_source_mappping = data;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn render_in_window(
        &mut self,
        rect_lo_x: f32,
        rect_lo_y: f32,
        rect_hi_x: f32,
        rect_hi_y: f32,
    ) -> String {
        let _rect = typst_ts_core::vector::geom::Rect {
            lo: Axes::new(Scalar(rect_lo_x), Scalar(rect_lo_y)),
            hi: Axes::new(Scalar(rect_hi_x), Scalar(rect_hi_y)),
        };

        let doc_view = self.doc_view.take();
        let next_doc_view = self.doc.layouts[0].1.clone();
        let res =
            IncrementalSvgV2Exporter::render_in_window(&self.doc.module, doc_view, &next_doc_view);
        self.doc_view = Some(next_doc_view);
        res
    }
}

#[wasm_bindgen]
impl TypstRenderer {
    pub fn create_session(
        &self,
        artifact_content: &[u8],
        options: Option<RenderSessionOptions>,
    ) -> ZResult<RenderSession> {
        self.session_mgr
            .create_session_internal(artifact_content, options)
    }

    pub fn create_svg_session(&self, artifact_content: &[u8]) -> ZResult<SvgSession> {
        Ok(SvgSession {
            doc: typst_ts_svg_exporter::MultiSvgDocument::from_slice(artifact_content),
            doc_view: None,
            source_mapping_data: Default::default(),
            page_source_mappping: Default::default(),
        })
    }

    pub fn create_empty_svg_session(&self) -> ZResult<SvgSession> {
        Ok(SvgSession {
            doc: typst_ts_svg_exporter::MultiSvgDocument::default(),
            doc_view: None,
            source_mapping_data: Default::default(),
            page_source_mappping: Default::default(),
        })
    }

    pub fn render_svg(
        &self,
        session: &mut SvgSession,
        root: web_sys::HtmlDivElement,
    ) -> ZResult<()> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        let layout = session.doc.layouts.first().unwrap();

        // base scale = 2
        let base_cw = root.client_width() as f32;

        let render = |layout: &LayoutElem| {
            let applying = format!("{}px", layout.0 .0);

            let applied = root.get_attribute("data-applyed-width");
            if applied.is_some() && applied.unwrap() == applying {
                console_log!("already applied {}", applying);
                return Ok(());
            }

            let svg = UsingExporter::render_flat_svg(&session.doc.module, &layout.1);
            root.set_inner_html(&svg);
            let window = web_sys::window().unwrap();
            if let Ok(proc) = js_sys::Reflect::get(&window, &JsValue::from_str("typstProcessSvg")) {
                web_sys::console::log_1(&proc);
                proc.dyn_ref::<js_sys::Function>()
                    .unwrap()
                    .call1(&JsValue::NULL, &root.first_element_child().unwrap())
                    .unwrap();
            }

            root.set_attribute("data-applyed-width", &applying).unwrap();
            console_log!("applied {}", applying);

            Ok(())
        };

        if layout.0 .0 < base_cw {
            return render(layout);
        }

        let layout = session.doc.layouts.last().unwrap();

        if layout.0 .0 > base_cw {
            return render(layout);
        }

        for layout in &session.doc.layouts {
            if layout.0 .0 < base_cw {
                return render(layout);
            }
        }

        Ok(())
    }

    pub fn load_page(
        &self,
        session: &mut RenderSession,
        page_number: usize,
        page_content: String,
    ) -> ZResult<()> {
        self.session_mgr
            .load_page(session, page_number, page_content)
    }
}

impl TypstRenderer {
    pub fn new(font_resolver: FontResolverImpl) -> TypstRenderer {
        Self {
            session_mgr: RenderSessionManager::new(font_resolver),
        }
    }

    fn retrieve_page_off(
        &self,
        ses: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<usize> {
        if ses.doc.pages.is_empty() {
            return Err(error_once!("Renderer.SessionDocNotPages"));
        }

        let page_off = options.as_ref().map(|o| o.page_off).unwrap_or(0);
        if page_off < ses.doc.pages.len() && page_off == ses.pages_info.pages[page_off].page_off {
            return Ok(page_off);
        }

        for (i, page_info) in ses.pages_info.pages.iter().enumerate() {
            if page_info.page_off == page_off {
                return Ok(i);
            }
        }

        Err(error_once!(
            "Renderer.SessionPageNotFound",
            offset: page_off
        ))
    }

    pub fn session_from_artifact(&self, artifact_content: &[u8]) -> ZResult<RenderSession> {
        self.session_mgr
            .session_from_artifact(artifact_content, "js")
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use typst_ts_core::Bytes;

    use super::{TypstRenderer, TypstRendererBuilder};
    use std::path::PathBuf;

    fn artifact_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fuzzers/corpora/math/main.artifact.json")
    }

    pub fn get_renderer() -> TypstRenderer {
        let mut root_path = PathBuf::new();
        root_path.push(".");

        let mut builder = TypstRendererBuilder::new().unwrap();

        // todo: prepare font files for test
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_R.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RB.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RBI.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RI.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/NewCMMath-Book.otf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/NewCMMath-Regular.otf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-Bold.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-BoldItalic.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-Italic.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-Regular.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/Roboto-Regular.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/NotoSerifCJKsc-Regular.otf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-Oblique.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-BoldOblique.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-Bold.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/TwitterColorEmoji.ttf"
        )));
        builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
            "../../../assets/fonts/NotoColorEmoji.ttf"
        )));

        pollster::block_on(builder.build()).unwrap()
    }

    #[test]
    fn test_render_document() {
        let renderer = get_renderer();

        let artifact_content = std::fs::read(artifact_path()).unwrap();

        let mut ses = renderer
            .session_mgr
            .session_from_artifact(artifact_content.as_slice(), "serde_json")
            .unwrap();
        ses.pixel_per_pt = 2.;
        ses.background_color = "ffffff".to_string();

        renderer.render_to_image_internal(&ses, None).unwrap();
    }
}
