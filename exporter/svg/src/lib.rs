//! Rendering into svg text or module.

use std::collections::HashMap;
use std::sync::Arc;

use ir::{DefId, FlatSvgItem, GroupRef, Module, ModuleBuilder, TransformItem};
pub(crate) use tiny_skia as sk;

use typst::diag::SourceResult;
use typst::doc::{Document, Frame};
use typst::font::FontInfo;
use typst::World;
use typst_ts_core::annotation::AnnotationList;
use typst_ts_core::error::prelude::*;
use typst_ts_core::font::{FontGlyphProvider, GlyphProvider};
use typst_ts_core::{Exporter, TextContent};
use utils::{AbsExt, PerfEvent};

pub(crate) mod annotation;
pub(crate) mod content;
pub(crate) mod image;
pub(crate) mod ir;
pub(crate) mod lowering;
pub(crate) mod shape;
pub(crate) mod svg;
pub(crate) mod text;
pub(crate) mod utils;

pub trait RenderFeature {
    const ENABLE_TRACING: bool;
}

pub struct DefaultRenderFeature;

impl RenderFeature for DefaultRenderFeature {
    const ENABLE_TRACING: bool = false;
}

pub struct SvgRenderTask<Feat: RenderFeature = DefaultRenderFeature> {
    glyph_provider: GlyphProvider,

    glyph_defs: HashMap<String, (String, u32)>,
    clip_paths: HashMap<String, u32>,

    module: Arc<Module>,

    page_off: usize,
    width_px: u32,
    height_px: u32,
    raw_height: f32,

    pub text_content: TextContent,
    pub annotations: AnnotationList,

    font_map: HashMap<FontInfo, u32>,

    // errors: Vec<Error>,
    _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    pub fn new() -> ZResult<Self> {
        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        Ok(Self {
            glyph_provider: default_glyph_provider,
            glyph_defs: HashMap::default(),
            clip_paths: HashMap::default(),

            module: Arc::new(Module::default()),
            page_off: 0,
            width_px: 0,
            height_px: 0,
            raw_height: 0.,

            text_content: TextContent::default(),
            annotations: AnnotationList::default(),
            font_map: HashMap::default(),

            _feat_phantom: Default::default(),
        })
    }

    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

    #[inline]
    fn perf_event(&self, _name: &'static str) -> Option<PerfEvent> {
        None
    }

    /// Directly render a frame into the canvas.
    pub fn render(&mut self, frame: &Frame) -> ZResult<String> {
        let item = self.lower(frame)?;
        let (entry, module) = item.flatten();
        self.module = Arc::new(module);

        let root = self.module.get_item(entry).cloned();
        if let Some(FlatSvgItem::Group(root)) = root {
            return self.render_frame(entry, &root);
        }

        Err(error_once!("SvgRenderTask.RootNotAGroup"))
    }

    /// Render a frame into the canvas.
    fn render_frame(&mut self, frame_id: DefId, frame: &GroupRef) -> ZResult<String> {
        let mut g = vec!["<g>".to_owned()];

        for (pos, item) in frame.0.iter() {
            g.push(format!(
                r#"<g transform="translate({},{})" >"#,
                pos.x.to_pt(),
                pos.y.to_pt()
            ));
            g.push(self.render_item(frame_id.make_absolute(*item))?);
            g.push("</g>".to_owned());
        }

        g.push("</g>".to_owned());

        Ok(g.join(""))
    }

    fn get_css(&mut self, transform: &TransformItem) -> String {
        match transform {
            TransformItem::Matrix(m) => {
                format!(
                    r#"transform="matrix({},{},{},{},{},{})""#,
                    m.sx.get(),
                    m.ky.get(),
                    m.kx.get(),
                    m.sy.get(),
                    m.tx.to_pt(),
                    m.ty.to_pt()
                )
            }
            // TransformItem::Translate(tx, ty) => format!("translate({},{})", tx, ty),
            // TransformItem::Scale(sx, sy) => format!("scale({},{})", sx, sy),
            // TransformItem::Rotate(angle) => format!("rotate({})", angle),
            // TransformItem::SkewX(angle) => format!("skewX({})", angle),
            // TransformItem::SkewY(angle) => format!("skewY({})", angle),
            TransformItem::Clip(c) => {
                let clip_id;
                if let Some(c) = self.clip_paths.get(&c.d) {
                    clip_id = *c;
                } else {
                    let cid = self.clip_paths.len() as u32;
                    self.clip_paths.insert(c.d.clone(), cid);
                    clip_id = cid;
                }

                format!(r##"clip-path="url(#c{:x})""##, clip_id)
            }
        }
    }

    fn render_item(&mut self, def_id: DefId) -> ZResult<String> {
        let item = self.module.get_item(def_id).unwrap().clone();
        match item {
            FlatSvgItem::Group(group) => self.render_frame(def_id, &group),
            FlatSvgItem::Text(text) => self.render_text(&text),
            FlatSvgItem::Path(path) => self.render_path(&path),
            FlatSvgItem::Item(transformed) => {
                let item = self.render_item(def_id.make_absolute(transformed.1))?;
                Ok(format!(
                    r#"<g {}>{}</g>"#,
                    self.get_css(&transformed.0),
                    item
                ))
            }
            FlatSvgItem::Image(image) => self.render_image(&image.image, image.size),
            FlatSvgItem::Glyph(_) | FlatSvgItem::None => {
                panic!("SvgRenderTask.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }
}

#[derive(Default)]
pub struct SvgExporter {}

impl Exporter<Document, String> for SvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        let header = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {:.3} {:.3}" >"#,
            // todo: without page
            output.pages[0].width().to_pt(),
            output.pages[0].height().to_pt() * output.pages.len() as f64,
        );
        let mut svg = vec![header];
        let mut svg_body = vec![];

        let mut t = SvgRenderTask::<DefaultRenderFeature>::new().unwrap();

        let mut acc_height = 0f32;
        for (idx, page) in output.pages.iter().enumerate() {
            let (width_px, height_px) = {
                let size = page.size();
                let width_px = (size.x.to_pt() as f32).round().max(1.0) as u32;
                let height_px = (size.y.to_pt() as f32).round().max(1.0) as u32;

                (width_px, height_px)
            };
            t.page_off = idx;
            t.width_px = width_px;
            t.height_px = height_px;
            t.raw_height = height_px as f32;
            let item = t.render(page).unwrap();
            let item = format!(
                r#"<g transform="translate(0, {})" >{}</g>"#,
                acc_height, item
            );

            svg_body.push(item);
            acc_height += height_px as f32;
        }

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
        svg.append(&mut svg_body);

        svg.push("</svg>".to_owned());
        let svg = svg.join("");
        Ok(svg)
    }
}

#[derive(Default)]
pub struct SvgModuleExporter {}

impl Exporter<Document, Vec<u8>> for SvgModuleExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<Vec<u8>> {
        let mut t = SvgRenderTask::<DefaultRenderFeature>::new().unwrap();

        let mut builder = ModuleBuilder::default();

        for page in output.pages.iter() {
            let item = t.lower(page).unwrap();
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
