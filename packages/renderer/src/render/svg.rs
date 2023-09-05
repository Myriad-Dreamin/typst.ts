use crate::{RenderSession, RenderSessionOptions, TypstRenderer};
use typst_ts_core::error::prelude::*;
use typst_ts_core::vector::geom::Axes;
use typst_ts_core::vector::geom::Scalar;
use typst_ts_svg_exporter::flat_ir::LayoutRegionNode;
use typst_ts_svg_exporter::flat_ir::SourceMappingNode;
use typst_ts_svg_exporter::IncrSvgDocClient;
use typst_ts_svg_exporter::{DefaultExportFeature, SvgExporter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SvgSession {
    client: IncrSvgDocClient,
}

fn access_slice<'a, T>(v: &'a [T], idx: usize, kind: &'static str, pos: usize) -> ZResult<&'a T> {
    v.get(idx).ok_or_else(
        || error_once!("out of bound access", pos: pos, kind: kind, idx: idx, actual: v.len()),
    )
}

#[wasm_bindgen]
impl SvgSession {
    pub fn reset(&mut self) {
        self.client = Default::default();
    }

    #[wasm_bindgen(getter)]
    pub fn doc_width(&self) -> f32 {
        if self.client.doc.layouts.is_empty() {
            return 0.;
        }

        // let pages = self.client.doc.layouts[0].iter();
        // pages.map(|(_, s)| s.x).max().unwrap_or_default().0
        todo!()
    }

    #[wasm_bindgen(getter)]
    pub fn doc_height(&self) -> f32 {
        if self.client.doc.layouts.is_empty() {
            return 0.;
        }

        // let pages = self.client.doc.layouts[0].iter();
        // pages.map(|(_, s)| s.y.0).sum()
        todo!()
    }

    pub fn merge_delta(&mut self, delta: &[u8]) -> ZResult<()> {
        use typst_ts_core::vector::stream::BytesModuleStream;

        let delta = BytesModuleStream::from_slice(delta).checkout_owned();

        #[cfg(feature = "debug_delta_update")]
        crate::utils::console_log!(
            "module counts: {:?},{:?},{:?}",
            delta.glyphs.len(),
            delta.item_pack.0.len(),
            delta.layouts.len()
        );

        self.client.merge_delta(delta);
        Ok(())
    }

    pub fn render_in_window(
        &mut self,
        rect_lo_x: f32,
        rect_lo_y: f32,
        rect_hi_x: f32,
        rect_hi_y: f32,
    ) -> String {
        use typst_ts_core::vector::geom::Rect;

        self.client.render_in_window(Rect {
            lo: Axes::new(Scalar(rect_lo_x), Scalar(rect_lo_y)),
            hi: Axes::new(Scalar(rect_hi_x), Scalar(rect_hi_y)),
        })
    }

    pub fn get_source_loc(&self, path: &[u32]) -> ZResult<Option<String>> {
        const SOURCE_MAPPING_TYPE_TEXT: u32 = 0;
        const SOURCE_MAPPING_TYPE_GROUP: u32 = 1;
        const SOURCE_MAPPING_TYPE_IMAGE: u32 = 2;
        const SOURCE_MAPPING_TYPE_SHAPE: u32 = 3;
        const SOURCE_MAPPING_TYPE_PAGE: u32 = 4;

        if self.client.page_source_mappping.is_empty() {
            return Ok(None);
        }

        let mut index_item: Option<&SourceMappingNode> = None;

        let source_mapping = self.client.source_mapping_data.as_slice();
        let page_sources = self.client.page_source_mappping[0]
            .source_mapping(&self.client.doc.module)
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
        use typst_ts_svg_exporter::MultiSvgDocument;

        let doc = MultiSvgDocument::from_slice(artifact_content);
        Ok(SvgSession {
            client: IncrSvgDocClient {
                doc,
                ..Default::default()
            },
        })
    }

    pub fn create_empty_svg_session(&self) -> ZResult<SvgSession> {
        Ok(SvgSession {
            client: Default::default(),
        })
    }

    pub fn render_svg(&self, session: &SvgSession, root: web_sys::HtmlDivElement) -> ZResult<()> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        let layouts = session.client.doc.layouts.by_scalar().unwrap();
        let layout = layouts.first().unwrap();

        // base scale = 2
        let base_cw = root.client_width() as f32;

        let render = |layout: &(Scalar, LayoutRegionNode)| {
            let applying = format!("{}px", layout.0 .0);

            let applied = root.get_attribute("data-applied-width");
            if applied.is_some() && applied.unwrap() == applying {
                // console_log!("already applied {}", applying);
                return Ok(());
            }

            let view = layout.1.pages(&session.client.doc.module).unwrap();

            let svg = UsingExporter::render_flat_svg(view.module(), view.pages());
            root.set_inner_html(&svg);
            let window = web_sys::window().unwrap();
            if let Ok(proc) = js_sys::Reflect::get(&window, &JsValue::from_str("typstProcessSvg")) {
                proc.dyn_ref::<js_sys::Function>()
                    .unwrap()
                    .call1(&JsValue::NULL, &root.first_element_child().unwrap())
                    .unwrap();
            }

            root.set_attribute("data-applied-width", &applying).unwrap();
            // console_log!("applied {}", applying);

            Ok(())
        };

        // console_log!("base_cw {}", base_cw);

        // console_log!(
        //     "layouts {:?}",
        //     session
        //         .client
        //         .doc
        //         .layouts
        //         .iter()
        //         .map(|x| x.0)
        //         .collect::<Vec<_>>()
        // );

        const EPS: f32 = 1e-2;

        if layout.0 .0 < base_cw + EPS {
            return render(layout);
        }

        let layout = layouts.last().unwrap();

        if layout.0 .0 + EPS > base_cw {
            return render(layout);
        }

        for layout in layouts {
            if layout.0 .0 < base_cw + EPS {
                return render(layout);
            }
        }

        Ok(())
    }
}
