//! Rendering into svg text or module.

use std::collections::HashMap;
use std::sync::Arc;

use ir::{Module, ModuleBuilder, StyleNs, TransformItem};
use render::SvgRenderTask;
pub(crate) use tiny_skia as sk;

use typst::diag::SourceResult;
use typst::doc::{Document, Frame};
use typst::geom::Axes;
use typst::World;
use typst_ts_core::annotation::link::AnnotationProcessor;
use typst_ts_core::annotation::AnnotationList;
use typst_ts_core::error::prelude::*;
use typst_ts_core::font::{FontGlyphProvider, GlyphProvider};
use typst_ts_core::{Exporter, TextContent};
use utils::AbsExt;

pub(crate) mod annotation;
pub(crate) mod content;
pub(crate) mod ir;
pub(crate) mod lowering;
pub(crate) mod render;
pub(crate) mod svg;
pub(crate) mod utils;

pub trait RenderFeature {
    const ENABLE_TRACING: bool;
}

pub struct DefaultRenderFeature;

impl RenderFeature for DefaultRenderFeature {
    const ENABLE_TRACING: bool = false;
}

pub struct SvgTask<Feat: RenderFeature = DefaultRenderFeature> {
    glyph_provider: GlyphProvider,
    annotation_proc: AnnotationProcessor,

    style_defs: HashMap<(StyleNs, Arc<str>), (String, u32)>,
    glyph_defs: HashMap<String, (String, u32)>,
    clip_paths: HashMap<String, u32>,

    pub text_content: TextContent,
    pub annotations: AnnotationList,

    // errors: Vec<Error>,
    _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: RenderFeature> SvgTask<Feat> {
    pub fn new(doc: &Document) -> ZResult<Self> {
        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        Ok(Self {
            glyph_provider: default_glyph_provider,
            annotation_proc: AnnotationProcessor::new(doc),
            style_defs: HashMap::default(),
            glyph_defs: HashMap::default(),
            clip_paths: HashMap::default(),

            text_content: TextContent::default(),
            annotations: AnnotationList::default(),

            _feat_phantom: Default::default(),
        })
    }

    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

    /// Directly render a frame into the canvas.
    pub fn render(&mut self, input: Arc<Document>, svg_body: &mut Vec<String>) -> ZResult<()> {
        let mut acc_height = 0f32;
        for (idx, page) in input.pages.iter().enumerate() {
            let (item, size) = self.render_frame(idx, page).unwrap();
            let item = format!(
                r#"<g transform="translate(0, {})" >{}</g>"#,
                acc_height, item
            );

            svg_body.push(item);
            acc_height += size.y as f32;
        }

        Ok(())
    }

    /// Directly render a frame into the canvas.
    pub fn render_frame(&mut self, idx: usize, frame: &Frame) -> ZResult<(String, Axes<u32>)> {
        let item = self.lower(frame);
        let (entry, module) = item.flatten();

        let (width_px, height_px) = {
            let size = frame.size();
            let width_px = (size.x.to_pt() as f32).round().max(1.0) as u32;
            let height_px = (size.y.to_pt() as f32).round().max(1.0) as u32;

            (width_px, height_px)
        };

        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        let mut t = SvgRenderTask::<DefaultRenderFeature> {
            glyph_provider: default_glyph_provider,

            module: &module,

            style_defs: &mut self.style_defs,
            glyph_defs: &mut self.glyph_defs,
            clip_paths: &mut self.clip_paths,
            text_content: &mut self.text_content,
            annotations: &mut self.annotations,

            page_off: idx,
            width_px,
            height_px,
            raw_height: height_px as f32,

            font_map: HashMap::default(),

            _feat_phantom: Default::default(),
        };

        Ok((t.render_item(entry)?, Axes::new(width_px, height_px)))
    }
}

#[derive(Default)]
pub struct SvgExporter {}

impl Exporter<Document, String> for SvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        // todo: without page
        let w = output.pages[0].width().to_pt();
        let h = output.pages[0].height().to_pt() * output.pages.len() as f64;
        let header = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" >"#,
            w, h, w, h,
        );
        let mut svg = vec![header];
        let mut svg_body = vec![];

        let mut t = SvgTask::<DefaultRenderFeature>::new(&output).unwrap();
        t.render(output, &mut svg_body).unwrap();

        svg.push("<defs>".to_owned());
        svg.push("<g>".to_owned());
        let mut g = t.glyph_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1 .1.cmp(&b.1 .1));
        for (_, (glyph, ..)) in g {
            svg.push(glyph);
        }
        svg.push("</g>".to_owned());

        let mut g = t.clip_paths.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in g {
            svg.push(format!(
                r##"<clipPath id="c{:x}"><path d="{}"/></clipPath>"##,
                id, clip_path
            ));
        }

        svg.push("</defs>".to_owned());
        // var elements = document.getElementsByClassName('childbox');
        // for(var i=0; i<elements.length; i++) {
        //   elements[i].onmouseleave = function(){
        //     this.style.fill = "blue";
        // };
        // }

        svg.push(r#"<style type="text/css">"#.to_owned());
        svg.push(
            r#"
        g.t { pointer-events: bounding-box; }
        svg { --glyph_fill: black; }
        .pseudo-link { fill: transparent; cursor: pointer; pointer-events: all; }
        .outline_glyph { fill: var(--glyph_fill); }
        .outline_glyph { transition: 0.2s all; }
        .hover .t { --glyph_fill: #66BAB7; }
        .t:hover { --glyph_fill: #F75C2F; }"#
                .replace("        ", ""),
        );
        let mut g = t.style_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1 .1.cmp(&b.1 .1));
        svg.extend(g.into_iter().map(|v| v.1 .0));
        svg.push("</style>".to_owned());
        svg.append(&mut svg_body);

        svg.push(r#"<script type="text/javascript">"#.to_owned());
        svg.push(r#"<![CDATA["#.to_owned());
        svg.push(String::from_utf8(include_bytes!("./typst.svg.js").to_vec()).unwrap());
        svg.push(r#"]]>"#.to_owned());
        svg.push("</script>".to_owned());
        svg.push("</svg>".to_owned());
        let svg = svg.join("");
        Ok(svg)
    }
}

#[derive(Default)]
pub struct SvgModuleExporter {}

impl Exporter<Document, Vec<u8>> for SvgModuleExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<Vec<u8>> {
        let mut t = SvgTask::<DefaultRenderFeature>::new(&output).unwrap();

        let mut builder = ModuleBuilder::default();

        for page in output.pages.iter() {
            let item = t.lower(page);
            let _entry_id = builder.build(item);
        }

        let mut res = vec![];
        let repr = builder.finalize();
        serialize_module(&mut res, repr);
        Ok(res)
    }
}

fn serialize_module(res: &mut Vec<u8>, repr: Module) {
    fn serialize_glyph(res: &mut Vec<u8>, item: &ir::GlyphItem) {
        match item {
            ir::GlyphItem::Raw(_font, id) => {
                res.push(b'r');
                res.extend_from_slice(&0u32.to_le_bytes());
                res.extend_from_slice(&id.0.to_le_bytes());
            }
        }
    }

    for k in repr.glyphs {
        serialize_glyph(res, &k)
    }

    for i in repr.items {
        match i {
            ir::FlatSvgItem::None => {
                res.push(b'0');
            }
            ir::FlatSvgItem::Glyph(id) => {
                res.push(b'g');
                serialize_glyph(res, id.as_ref())
            }
            ir::FlatSvgItem::Image(id) => {
                res.push(b'i');
                res.extend_from_slice(&id.size.x.to_f32().to_le_bytes());
                res.extend_from_slice(&id.size.y.to_f32().to_le_bytes());
                // todo: image
                res.extend_from_slice(id.image.data());
            }
            ir::FlatSvgItem::Link(id) => {
                res.push(b'l');
                res.extend_from_slice(id.href.as_bytes());
                res.extend_from_slice(&id.size.x.to_f32().to_le_bytes());
                res.extend_from_slice(&id.size.y.to_f32().to_le_bytes());
                for a in &id.affects {
                    res.extend_from_slice(&a.0.to_le_bytes());
                }
            }
            ir::FlatSvgItem::Path(id) => {
                res.push(b'p');
                res.extend_from_slice(id.d.as_bytes());
                for s in &id.styles {
                    match s {
                        ir::PathStyle::Fill(id) => {
                            res.push(b'f');
                            res.extend_from_slice(id.as_bytes());
                        }
                        ir::PathStyle::Stroke(id) => {
                            res.push(b's');
                            res.extend_from_slice(id.as_bytes());
                        }
                        ir::PathStyle::StrokeLineCap(id) => {
                            res.push(b'c');
                            res.extend_from_slice(id.as_bytes());
                        }
                        ir::PathStyle::StrokeLineJoin(id) => {
                            res.push(b'j');
                            res.extend_from_slice(id.as_bytes());
                        }
                        ir::PathStyle::StrokeMitterLimit(id) => {
                            res.push(b'm');
                            res.extend_from_slice(&(id.0 as f32).to_le_bytes());
                        }
                        ir::PathStyle::StrokeDashOffset(id) => {
                            res.push(b'o');
                            res.extend_from_slice(&id.to_f32().to_le_bytes());
                        }
                        ir::PathStyle::StrokeDashArray(id) => {
                            res.push(b'a');
                            res.extend_from_slice(&id.len().to_le_bytes());
                            for i in id.iter() {
                                res.extend_from_slice(&i.to_f32().to_le_bytes());
                            }
                        }
                        ir::PathStyle::StrokeWidth(id) => {
                            res.push(b'w');
                            res.extend_from_slice(&id.to_f32().to_le_bytes());
                        }
                    }
                }
            }
            ir::FlatSvgItem::Text(id) => {
                res.push(b't');
                for g in &id.glyphs {
                    res.push(b'g');
                    res.extend_from_slice(&g.0.to_le_bytes());
                }
                res.extend_from_slice(id.content.as_bytes());
                res.extend_from_slice(id.shape.fill.as_bytes());
            }
            ir::FlatSvgItem::Item(id) => {
                res.push(b'i');
                match &id.0 {
                    TransformItem::Clip(p) => {
                        res.push(b'c');
                        res.extend_from_slice(p.d.as_bytes());
                    }
                    TransformItem::Matrix(p) => {
                        res.push(b'm');
                        res.extend_from_slice(&(p.sx.get() as f32).to_le_bytes());
                        res.extend_from_slice(&(p.ky.get() as f32).to_le_bytes());
                        res.extend_from_slice(&(p.kx.get() as f32).to_le_bytes());
                        res.extend_from_slice(&(p.sy.get() as f32).to_le_bytes());
                        res.extend_from_slice(&p.tx.to_f32().to_le_bytes());
                        res.extend_from_slice(&p.ty.to_f32().to_le_bytes());
                    }
                    TransformItem::Translate(t) => {
                        res.push(b't');
                        res.extend_from_slice(&t.x.to_f32().to_le_bytes());
                        res.extend_from_slice(&t.y.to_f32().to_le_bytes());
                    }
                    TransformItem::Scale(t) => {
                        let (sx, sy) = t.as_ref();
                        res.push(b's');
                        res.extend_from_slice(&(sx.get() as f32).to_le_bytes());
                        res.extend_from_slice(&(sy.get() as f32).to_le_bytes());
                    }
                    TransformItem::Rotate(t) => {
                        let r = t.as_ref();
                        res.push(b'r');
                        res.extend_from_slice(&(r.0 as f32).to_le_bytes());
                    }
                    TransformItem::Skew(t) => {
                        let (kx, ky) = t.as_ref();
                        res.push(b'k');
                        res.extend_from_slice(&(kx.get() as f32).to_le_bytes());
                        res.extend_from_slice(&(ky.get() as f32).to_le_bytes());
                    }
                }
                res.extend_from_slice(&id.1 .0.to_le_bytes());
            }
            ir::FlatSvgItem::Group(id) => {
                res.push(b'g');
                for item in id.0.iter() {
                    res.extend_from_slice(&item.0.x.to_f32().to_le_bytes());
                    res.extend_from_slice(&item.0.y.to_f32().to_le_bytes());
                    res.extend_from_slice(&item.1 .0.to_le_bytes());
                }
            }
        }
    }
}
