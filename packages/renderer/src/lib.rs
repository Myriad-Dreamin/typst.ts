// #[macro_use]
// pub(crate) mod utils;

// use typst_ts_core::error::prelude::*;
// use typst_ts_core::font::FontResolverImpl;
// use typst_ts_core::vector::geom::Axes;
// use typst_ts_core::vector::geom::Scalar;
// use typst_ts_svg_exporter::flat_ir::SourceMappingNode;
// use typst_ts_svg_exporter::IncrSvgDocClient;
// use typst_ts_svg_exporter::{DefaultExportFeature, LayoutElem, SvgExporter};
// use wasm_bindgen::prelude::*;

// pub(crate) mod parser;

// pub(crate) mod builder;
// pub use builder::TypstRendererBuilder;

// pub(crate) mod render;

// pub(crate) mod session;
// pub use session::RenderSession;

// pub use session::{RenderSessionManager, RenderSessionOptions};

// pub mod build_info {
//     /// The version of the typst-ts-renderer crate.
//     pub static VERSION: &str = env!("CARGO_PKG_VERSION");

//     /// The features of the typst-ts-renderer crate.
//     pub static FEATURES: &str = env!("VERGEN_CARGO_FEATURES");

//     /// The commit hash of the typst-ts-renderer crate.
//     pub static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");

//     /// The profile of the typst-ts-renderer crate.
//     /// It should be typically "debug" or "release". It is specifically
// exactly     /// the value passed by `cargo build --profile $VALUE`.
//     pub static PROFILE: &str = env!("VERGEN_CARGO_PROFILE");

//     pub fn features() -> Vec<&'static str> {
//         FEATURES.split(',').collect::<Vec<_>>()
//     }
// }

// #[wasm_bindgen]
// pub fn renderer_build_info() -> JsValue {
//     let obj = js_sys::Object::new();

//     js_sys::Reflect::set(
//         &obj,
//         &JsValue::from_str("version"),
//         &JsValue::from_str(build_info::VERSION),
//     )
//     .unwrap();

//     js_sys::Reflect::set(
//         &obj,
//         &JsValue::from_str("features"),
//         &build_info::features()
//             .into_iter()
//             .map(JsValue::from_str)
//             .collect::<js_sys::Array>(),
//     )
//     .unwrap();

//     js_sys::Reflect::set(
//         &obj,
//         &JsValue::from_str("commit_hash"),
//         &JsValue::from_str(build_info::COMMIT_HASH),
//     )
//     .unwrap();

//     js_sys::Reflect::set(
//         &obj,
//         &JsValue::from_str("profile"),
//         &JsValue::from_str(build_info::PROFILE),
//     )
//     .unwrap();

//     obj.into()
// }

// #[wasm_bindgen]
// #[derive(Debug, Default)]
// pub struct RenderPageImageOptions {
//     pub(crate) page_off: usize,
// }

// #[wasm_bindgen]
// impl RenderPageImageOptions {
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> Self {
//         Self { page_off: 0 }
//     }

//     #[wasm_bindgen(getter)]
//     pub fn page_off(&self) -> usize {
//         self.page_off
//     }

//     #[wasm_bindgen(setter)]
//     pub fn set_page_off(&mut self, page_off: usize) {
//         self.page_off = page_off;
//     }
// }

// #[wasm_bindgen]
// pub struct TypstRenderer {
//     pub(crate) session_mgr: RenderSessionManager,
// }

// #[wasm_bindgen]
// pub struct SvgSession {
//     client: IncrSvgDocClient,
// }

// fn access_slice<'a, T>(v: &'a [T], idx: usize, kind: &'static str, pos:
// usize) -> ZResult<&'a T> {     v.get(idx).ok_or_else(
//         || error_once!("out of bound access", pos: pos, kind: kind, idx: idx, actual: v.len()),
//     )
// }

// #[wasm_bindgen]
// impl SvgSession {
//     pub fn reset(&mut self) {
//         self.client = Default::default();
//     }

//     #[wasm_bindgen(getter)]
//     pub fn doc_width(&self) -> f32 {
//         if self.client.doc.layouts.is_empty() {
//             return 0.;
//         }

//         let pages = self.client.doc.layouts[0].iter();
//         pages.map(|(_, s)| s.x).max().unwrap_or_default().0
//     }

//     #[wasm_bindgen(getter)]
//     pub fn doc_height(&self) -> f32 {
//         if self.client.doc.layouts.is_empty() {
//             return 0.;
//         }

//         let pages = self.client.doc.layouts[0].iter();
//         pages.map(|(_, s)| s.y.0).sum()
//     }

//     pub fn merge_delta(&mut self, delta: &[u8]) -> ZResult<()> {
//         use typst_ts_core::vector::stream::BytesModuleStream;

//         let delta = BytesModuleStream::from_slice(delta).checkout_owned();

//         #[cfg(feature = "debug_delta_update")]
//         crate::utils::console_log!(
//             "module counts: {:?},{:?},{:?}",
//             delta.glyphs.len(),
//             delta.item_pack.0.len(),
//             delta.layouts.len()
//         );

//         self.client.merge_delta(delta);
//         Ok(())
//     }

//     pub fn render_in_window(
//         &mut self,
//         rect_lo_x: f32,
//         rect_lo_y: f32,
//         rect_hi_x: f32,
//         rect_hi_y: f32,
//     ) -> String { use typst_ts_core::vector::geom::Rect;

//         self.client.render_in_window(Rect {
//             lo: Axes::new(Scalar(rect_lo_x), Scalar(rect_lo_y)),
//             hi: Axes::new(Scalar(rect_hi_x), Scalar(rect_hi_y)),
//         })
//     }

//     pub fn get_source_loc(&self, path: &[u32]) -> ZResult<Option<String>> {
//         const SOURCE_MAPPING_TYPE_TEXT: u32 = 0;
//         const SOURCE_MAPPING_TYPE_GROUP: u32 = 1;
//         const SOURCE_MAPPING_TYPE_IMAGE: u32 = 2;
//         const SOURCE_MAPPING_TYPE_SHAPE: u32 = 3;
//         const SOURCE_MAPPING_TYPE_PAGE: u32 = 4;

//         if self.client.page_source_mappping.is_empty() {
//             return Ok(None);
//         }

//         let mut index_item: Option<&SourceMappingNode> = None;

//         let source_mapping = self.client.source_mapping_data.as_slice();
//         let page_sources = self.client.page_source_mappping[0].as_slice();

//         for (chunk_idx, v) in path.chunks_exact(2).enumerate() {
//             let (ty, idx) = (v[0], v[1] as usize);

//             let this_item = match index_item {
//                 Some(SourceMappingNode::Group(q)) => {
//                     let idx = *access_slice(q, idx, "group_index",
// chunk_idx)? as usize;                     access_slice(source_mapping, idx,
// "source_mapping", chunk_idx)?                 }
//                 Some(_) => {
//                     return Err(
//                         error_once!("cannot index", pos: chunk_idx, indexing:
// format!("{:?}", index_item)),                     )
//                 }
//                 None => access_slice(page_sources, idx, "page_sources",
// chunk_idx)?,             };

//             match (ty, this_item) {
//                 (SOURCE_MAPPING_TYPE_PAGE,
// SourceMappingNode::Page(page_index)) => {                     index_item =
// Some(access_slice(                         source_mapping,
//                         *page_index as usize,
//                         "source_mapping",
//                         chunk_idx,
//                     )?);
//                 }
//                 (SOURCE_MAPPING_TYPE_GROUP, SourceMappingNode::Group(_)) => {
//                     index_item = Some(this_item);
//                 }
//                 (SOURCE_MAPPING_TYPE_TEXT, SourceMappingNode::Text(n))
//                 | (SOURCE_MAPPING_TYPE_IMAGE, SourceMappingNode::Image(n))
//                 | (SOURCE_MAPPING_TYPE_SHAPE, SourceMappingNode::Shape(n)) =>
// {                     return Ok(Some(format!("{n:x}")));
//                 }
//                 _ => {
//                     return Err(error_once!("invalid/mismatch node type",
//                         pos: chunk_idx, ty: ty,
//                         actual: format!("{:?}", this_item),
//                         parent: format!("{:?}", index_item),
//                         child_idx_in_parent: idx,
//                     ))
//                 }
//             }
//         }

//         Ok(None)
//     }
// }

// #[wasm_bindgen]
// impl TypstRenderer {
//     pub fn create_session(
//         &self,
//         artifact_content: &[u8],
//         options: Option<RenderSessionOptions>,
//     ) -> ZResult<RenderSession> { self.session_mgr
//       .create_session_internal(artifact_content, options)
//     }

//     pub fn create_svg_session(&self, artifact_content: &[u8]) ->
// ZResult<SvgSession> {         use typst_ts_svg_exporter::MultiSvgDocument;

//         let doc = MultiSvgDocument::from_slice(artifact_content);
//         Ok(SvgSession {
//             client: IncrSvgDocClient {
//                 doc,
//                 ..Default::default()
//             },
//         })
//     }

//     pub fn create_empty_svg_session(&self) -> ZResult<SvgSession> {
//         Ok(SvgSession {
//             client: Default::default(),
//         })
//     }

//     pub fn render_svg(
//         &self,
//         session: &mut SvgSession,
//         root: web_sys::HtmlDivElement,
//     ) -> ZResult<()> { type UsingExporter =
//       SvgExporter<DefaultExportFeature>; let layout =
//       session.client.doc.layouts.first().unwrap();

//         // base scale = 2
//         let base_cw = root.client_width() as f32;

//         let render = |layout: &LayoutElem| {
//             let applying = format!("{}px", layout.0 .0);

//             let applied = root.get_attribute("data-applied-width");
//             if applied.is_some() && applied.unwrap() == applying {
//                 // console_log!("already applied {}", applying);
//                 return Ok(());
//             }

//             let svg =
// UsingExporter::render_flat_svg(&session.client.doc.module, &layout.1);
//             root.set_inner_html(&svg);
//             let window = web_sys::window().unwrap();
//             if let Ok(proc) = js_sys::Reflect::get(&window,
// &JsValue::from_str("typstProcessSvg")) {                 
// proc.dyn_ref::<js_sys::Function>()                     .unwrap()
//                     .call1(&JsValue::NULL,
// &root.first_element_child().unwrap())                     .unwrap();
//             }

//             root.set_attribute("data-applied-width", &applying).unwrap();
//             // console_log!("applied {}", applying);

//             Ok(())
//         };

//         // console_log!("base_cw {}", base_cw);

//         // console_log!(
//         //     "layouts {:?}",
//         //     session
//         //         .client
//         //         .doc
//         //         .layouts
//         //         .iter()
//         //         .map(|x| x.0)
//         //         .collect::<Vec<_>>()
//         // );

//         const EPS: f32 = 1e-2;

//         if layout.0 .0 < base_cw + EPS {
//             return render(layout);
//         }

//         let layout = session.client.doc.layouts.last().unwrap();

//         if layout.0 .0 + EPS > base_cw {
//             return render(layout);
//         }

//         for layout in &session.client.doc.layouts {
//             if layout.0 .0 < base_cw + EPS {
//                 return render(layout);
//             }
//         }

//         Ok(())
//     }

//     pub fn load_page(
//         &self,
//         session: &mut RenderSession,
//         page_number: usize,
//         page_content: String,
//     ) -> ZResult<()> { self.session_mgr .load_page(session, page_number,
//       page_content)
//     }
// }

// impl TypstRenderer {
//     pub fn new(font_resolver: FontResolverImpl) -> TypstRenderer {
//         Self {
//             session_mgr: RenderSessionManager::new(font_resolver),
//         }
//     }

//     fn retrieve_page_off(
//         &self,
//         ses: &RenderSession,
//         options: Option<RenderPageImageOptions>,
//     ) -> ZResult<usize> { if ses.doc.pages.is_empty() { return
//       Err(error_once!("Renderer.SessionDocNotPages")); }

//         let page_off = options.as_ref().map(|o| o.page_off).unwrap_or(0);
//         if page_off < ses.doc.pages.len() && page_off ==
// ses.pages_info.pages[page_off].page_off {             return Ok(page_off);
//         }

//         for (i, page_info) in ses.pages_info.pages.iter().enumerate() {
//             if page_info.page_off == page_off {
//                 return Ok(i);
//             }
//         }

//         Err(error_once!(
//             "Renderer.SessionPageNotFound",
//             offset: page_off
//         ))
//     }

//     pub fn session_from_artifact(&self, artifact_content: &[u8]) ->
// ZResult<RenderSession> {         self.session_mgr
//             .session_from_artifact(artifact_content, "js")
//     }
// }

// #[cfg(test)]
// #[cfg(target_arch = "wasm32")]
// mod tests {
//     use typst_ts_core::Bytes;

//     use super::{TypstRenderer, TypstRendererBuilder};
//     use std::path::PathBuf;

//     fn artifact_path() -> PathBuf {
//         PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//             .join("../../fuzzers/corpora/math/main.artifact.json")
//     }

//     pub fn get_renderer() -> TypstRenderer {
//         let mut root_path = PathBuf::new();
//         root_path.push(".");

//         let mut builder = TypstRendererBuilder::new().unwrap();

//         // todo: prepare font files for test
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/LinLibertine_R.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/LinLibertine_RB.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/LinLibertine_RBI.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/LinLibertine_RI.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/NewCMMath-Book.otf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/NewCMMath-Regular.otf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/InriaSerif-Bold.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/InriaSerif-BoldItalic.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/InriaSerif-Italic.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/InriaSerif-Regular.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/Roboto-Regular.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/NotoSerifCJKsc-Regular.otf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/DejaVuSansMono.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/DejaVuSansMono-Oblique.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/DejaVuSansMono-BoldOblique.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/DejaVuSansMono-Bold.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/TwitterColorEmoji.ttf"
//         )));
//         builder.add_raw_font_internal(Bytes::from_static(include_bytes!(
//             "../../../assets/fonts/NotoColorEmoji.ttf"
//         )));

//         pollster::block_on(builder.build()).unwrap()
//     }

//     #[test]
//     fn test_render_document() {
//         let renderer = get_renderer();

//         let artifact_content = std::fs::read(artifact_path()).unwrap();

//         let mut ses = renderer
//             .session_mgr
//             .session_from_artifact(artifact_content.as_slice(), "serde_json")
//             .unwrap();
//         ses.pixel_per_pt = 2.;
//         ses.background_color = "ffffff".to_string();

//         renderer.render_to_image_internal(&ses, None).unwrap();
//     }
// }

// todo
