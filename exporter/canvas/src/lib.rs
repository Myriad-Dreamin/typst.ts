//! Rendering into web_sys::CanvasRenderingContext2d.

use std::collections::HashMap;
use std::sync::Arc;

use svg::SvgPath2DBuilder;
pub(crate) use tiny_skia as sk;

use typst::doc::{Frame, FrameItem, GroupItem, Meta};
use typst::font::FontInfo;
use typst::geom::Color;
use typst_ts_core::error::prelude::*;
use typst_ts_core::font::{FontGlyphProvider, GlyphProvider};
use typst_ts_core::{Error, TextContent};
use utils::{js_random64, AbsExt, CanvasStateGuard, PerfEvent, ToCssExt};
use web_sys::{window, CanvasRenderingContext2d, Performance};

pub(crate) mod content;
pub(crate) mod utils;
pub(crate) use content::*;
pub(crate) mod image;
pub(crate) mod shape;
pub(crate) mod svg;
pub(crate) mod text;

pub trait RenderFeature {
    const ENABLE_TRACING: bool;
}

pub struct DefaultRenderFeature;

impl RenderFeature for DefaultRenderFeature {
    const ENABLE_TRACING: bool = false;
}

pub struct CanvasRenderTask<'a, Feat: RenderFeature = DefaultRenderFeature> {
    canvas: &'a CanvasRenderingContext2d,
    glyph_provider: GlyphProvider,

    pixel_per_pt: f32,
    fill: Color,

    width_px: u32,
    height_px: u32,
    raw_height: f32,

    pub content: TextContent,

    font_map: HashMap<FontInfo, u32>,

    perf: Option<Arc<Performance>>,
    perf_events: Option<&'a elsa::FrozenMap<&'static str, Box<f64>>>,
    errors: Vec<Error>,

    _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<'a, Feat: RenderFeature> CanvasRenderTask<'a, Feat> {
    pub fn new(
        canvas: &'a CanvasRenderingContext2d,
        doc: &'a typst::doc::Document,
        page_off: usize,
        pixel_per_pt: f32,
        fill: Color,
    ) -> ZResult<Self> {
        if pixel_per_pt <= 0. {
            return Err(error_once!(
                "CanvasRenderTask.InvalidPixelPerPt",
                pixel_per_pt: pixel_per_pt
            ));
        }

        let (width_px, height_px) = {
            let size = doc.pages[page_off].size();
            let width_px = (pixel_per_pt * (size.x.to_pt() as f32)).round().max(1.0) as u32;
            let height_px = (pixel_per_pt * (size.y.to_pt() as f32)).round().max(1.0) as u32;

            (width_px, height_px)
        };

        let session_id = format!("{:x}", js_random64());

        let canvas_ref = canvas
            .canvas()
            .ok_or_else(|| error_once!("CanvasRenderTask.GetCanvasRef"))?;
        canvas_ref
            .set_attribute("data-typst-session", &session_id)
            .map_err(map_err("CanvasRenderTask.SetDataTypstSessionId"))?;

        let perf = Feat::ENABLE_TRACING
            .then(|| window().and_then(|w| w.performance()))
            .flatten()
            .map(Arc::new);

        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        Ok(Self {
            canvas,
            glyph_provider: default_glyph_provider,

            pixel_per_pt,
            fill,

            width_px,
            height_px,
            raw_height: height_px as f32 / pixel_per_pt,

            content: TextContent::default(),
            font_map: HashMap::default(),
            errors: Vec::default(),

            _feat_phantom: Default::default(),
            perf,
            perf_events: None,
        })
    }

    pub fn set_perf_events(&mut self, perf_events: &'a elsa::FrozenMap<&'static str, Box<f64>>) {
        self.perf_events = Some(perf_events);
    }

    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

    #[inline]
    fn reset_transform(&mut self) {
        let maybe_err = self
            .canvas
            .reset_transform()
            .map_err(map_err("CanvasRenderTask.ResetTransform"));
        self.collect_err(maybe_err);
    }

    #[inline]
    fn sync_transform(&mut self, transform: sk::Transform) {
        // [ a c e ]
        // [ b d f ]
        // [ 0 0 1 ]

        // horizontal scaling
        let a = transform.sx as f64;
        // horizontal skewing
        let b = transform.ky as f64;
        // vertical skewing
        let c = transform.kx as f64;
        // vertical scaling
        let d = transform.sy as f64;
        // horizontal moving
        let e = transform.tx as f64;
        // vertical moving
        let f = transform.ty as f64;

        let maybe_err = self
            .canvas
            .transform(a, b, c, d, e, f)
            .map_err(map_err("CanvasRenderTask.SyncTransform"));
        self.collect_err(maybe_err);
    }

    #[inline]
    fn set_transform(&mut self, transform: sk::Transform) {
        // see sync_transform
        let a = transform.sx as f64;
        let b = transform.ky as f64;
        let c = transform.kx as f64;
        let d = transform.sy as f64;
        let e = transform.tx as f64;
        let f = transform.ty as f64;

        let maybe_err = self
            .canvas
            .set_transform(a, b, c, d, e, f)
            .map_err(map_err("CanvasRenderTask.SetTransform"));
        self.collect_err(maybe_err);
    }

    fn collect_err<T>(&mut self, maybe_err: Result<T, typst_ts_core::Error>) -> Option<T> {
        match maybe_err {
            Ok(v) => Some(v),
            Err(err) => {
                self.errors.push(err);
                None
            }
        }
    }

    #[inline]
    fn perf_event(&self, name: &'static str) -> Option<PerfEvent<'a>> {
        Feat::ENABLE_TRACING
            .then(|| {
                self.perf.as_ref().and_then(|perf| {
                    self.perf_events
                        .map(|pe| PerfEvent::new(name, perf.clone(), pe))
                })
            })
            .flatten()
    }

    /// Directly render a frame into the canvas.
    pub async fn render(&mut self, frame: &Frame) -> ZResult<()> {
        self.canvas.set_fill_style(&self.fill.to_css().into());
        self.canvas
            .fill_rect(0., 0., self.width_px as f64, self.height_px as f64);

        let ts = sk::Transform::from_scale(self.pixel_per_pt, self.pixel_per_pt);
        self.render_frame(ts, frame).await
    }

    /// Render a frame into the canvas.
    #[async_recursion::async_recursion(?Send)]
    async fn render_frame(&mut self, ts: sk::Transform, frame: &Frame) -> ZResult<()> {
        let mut text_flow = TextFlow::new();

        for (pos, item) in frame.items() {
            let x = pos.x.to_f32();
            let y = pos.y.to_f32();
            let ts = ts.pre_translate(x, y);

            match item {
                FrameItem::Group(group) => {
                    self.render_group(ts, group).await?;
                }
                FrameItem::Text(text) => {
                    let (next_text_flow, has_eol) = TextFlow::notify(text_flow, &ts, text);
                    text_flow = next_text_flow;

                    // has end of line (concept from pdf.js)
                    if has_eol {
                        self.append_text_break(ts, text)
                    }

                    self.render_text(ts, text).await;
                }
                FrameItem::Shape(shape, _) => {
                    self.render_shape(ts, shape)?;
                }
                FrameItem::Image(image, size, _) => {
                    self.render_image(ts, image, *size).await;
                }
                FrameItem::Meta(meta, _) => match meta {
                    Meta::Link(_) => {}
                    Meta::Elem(_) => {}
                    Meta::PageNumbering(_) => {}
                    Meta::Hide => {}
                },
            }
        }

        Ok(())
    }

    /// Render a group frame with optional transform and clipping into the canvas.
    #[async_recursion::async_recursion(?Send)]
    async fn render_group(&mut self, ts: sk::Transform, group: &GroupItem) -> ZResult<()> {
        let ts = ts.pre_concat(group.transform.into());

        let _clip_guard = if group.clips {
            let mask_box = {
                let mut builder = SvgPath2DBuilder::default();

                // build a rectangle path
                let size = group.frame.size();
                let w = size.x.to_f32();
                let h = size.y.to_f32();
                builder.rect(0., 0., w, h);

                builder
                    .build()
                    .map_err(map_err("CanvasRenderTask.BuildClip"))?
            };

            let guard = CanvasStateGuard::new(self.canvas);

            self.set_transform(ts);
            self.canvas.clip_with_path_2d(&mask_box);

            Some(guard)
        } else {
            None
        };

        self.render_frame(ts, &group.frame).await
    }
}
