#![allow(unused)]

use async_trait::async_trait;
use reflexo_vec2bbox::Vec2BBoxPass;

use crate::{utils::EmptyFuture, CanvasDevice, CanvasPaint};
use ecow::EcoVec;

use std::{
    cell::OnceCell,
    fmt::{self, Debug, Formatter},
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use js_sys::Promise;
use tiny_skia as sk;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasWindingRule, ImageBitmap, OffscreenCanvas, Path2d};

use reflexo::vector::ir::{
    self, FlatGlyphItem, Image, ImageItem, ImmutStr, PathStyle, Rect, Scalar,
};

use super::{rasterize_image, set_transform, BBoxAt, CanvasBBox, CanvasStateGuard};

#[derive(Default)]
pub struct CachedPath2d(OnceCell<Path2d>);

impl CachedPath2d {
    fn get_or_init(&self, d: &str) -> &Path2d {
        self.0
            .get_or_init(|| Path2d::new_with_path_string(d).unwrap())
    }
}

impl Debug for CachedPath2d {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachedPath2d")
            .field("initialized", &self.0.get().is_some())
            .finish()
    }
}

/// A reference to a canvas element.
pub type CanvasNode = Arc<CanvasElem>;
/// 2d Context
type Context2d = web_sys::CanvasRenderingContext2d;

/// The trait for all the operations that can be performed on some canvas
/// element.
#[async_trait(?Send)]
pub trait CanvasOp {
    /// Prepares the resource (recursively) for the action.
    fn prepare(
        &self,
        ts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + Sized + 'static>;
    /// Realizes the action on the canvas.
    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice);
}

/// A static enum for all the canvas elements.
#[derive(Debug)]
pub enum CanvasElem {
    /// A group of canvas elements.
    Group(CanvasGroupElem),
    /// references a canvas element with a clip path.
    Clip(CanvasClipElem),
    /// A path element.
    Path(CanvasPathElem),
    /// An image element.
    Image(CanvasImageElem),
    /// A glyph element.
    Glyph(CanvasGlyphElem),
}

#[async_trait(?Send)]
impl CanvasOp for CanvasElem {
    fn prepare(
        &self,
        ts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + Sized + 'static> {
        type DynFutureBox = Pin<Box<dyn core::future::Future<Output = ()>>>;

        match self {
            CanvasElem::Group(g) => g.prepare(ts).map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Clip(g) => g.prepare(ts).map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Path(g) => g.prepare(ts).map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Image(g) => g.prepare(ts).map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Glyph(g) => g.prepare(ts).map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
        }
    }

    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice) {
        match self {
            CanvasElem::Group(g) => g.realize(ts, canvas).await,
            CanvasElem::Clip(g) => g.realize(ts, canvas).await,
            CanvasElem::Path(g) => g.realize(ts, canvas).await,
            CanvasElem::Image(g) => g.realize(ts, canvas).await,
            CanvasElem::Glyph(g) => g.realize(ts, canvas).await,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GroupKind {
    General,
    Text,
}

/// A group of canvas elements.
#[derive(Debug)]
pub struct CanvasGroupElem {
    pub ts: Box<sk::Transform>,
    pub inner: EcoVec<(ir::Point, CanvasNode)>,
    pub kind: GroupKind,
    pub rect: CanvasBBox,
}

#[async_trait(?Send)]
impl CanvasOp for CanvasGroupElem {
    fn prepare(
        &self,
        rts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + Sized + 'static> {
        let mut v = Vec::default();

        for (_, sub_elem) in &self.inner {
            if let Some(f) = sub_elem.prepare(rts) {
                v.push(f);
            }
        }

        if v.is_empty() {
            None
        } else {
            Some(async move {
                for f in v {
                    f.await;
                }
            })
        }
    }

    async fn realize(&self, rts: sk::Transform, canvas: &dyn CanvasDevice) {
        let ts = rts.pre_concat(*self.ts.as_ref());

        for (pos, sub_elem) in &self.inner {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);
            sub_elem.realize(ts, canvas).await;
        }

        let _ = self.rect;
        let _ = Self::bbox_at;
        #[cfg(feature = "report_group")]
        web_sys::console::log_1(
            &format!("realize group {:?}({} elems)", self.kind, self.inner.len()).into(),
        );

        #[cfg(feature = "render_bbox")]
        {
            // realize bbox
            let bbox = self.bbox_at(rts);
            let color = if matches!(self.kind, GroupKind::Text) {
                "red"
            } else {
                "green"
            };

            render_bbox(canvas, bbox, color);

            #[cfg(feature = "report_bbox")]
            web_sys::console::log_1(&format!("realize group bbox {:?} {:?}", ts, bbox).into());
        }
    }
}

/// A reference to a canvas element with a clip path.
#[derive(Debug)]
pub struct CanvasClipElem {
    pub d: ImmutStr,
    pub inner: CanvasNode,
    pub clip_bbox: CanvasBBox,
    pub path: CachedPath2d,
}

impl CanvasClipElem {
    pub fn clip_bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        self.clip_bbox
            .bbox_at(ts, || Vec2BBoxPass::simple_path_bbox(&self.d, ts))
    }

    pub fn realize_with<'a>(
        &self,
        ts: sk::Transform,
        canvas: &'a dyn CanvasDevice,
    ) -> CanvasStateGuard<'a> {
        let guard = CanvasStateGuard::new(canvas);

        if !set_transform(canvas, ts) {
            return guard;
        }
        canvas.clip_with_path_2d(self.path.get_or_init(&self.d));

        guard
    }
}

#[async_trait(?Send)]
impl CanvasOp for CanvasClipElem {
    fn prepare(
        &self,
        ts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + Sized + 'static> {
        self.inner.prepare(ts)
    }

    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice) {
        let _guard = self.realize_with(ts, canvas);

        self.inner.realize(ts, canvas).await
    }
}

/// A path element.
#[derive(Debug)]
pub struct CanvasPathElem {
    pub path_data: Box<ir::PathItem>,
    pub fill: Option<CanvasPaint>,
    pub stroke: Option<CanvasPaint>,
    pub rect: CanvasBBox,
    pub path: CachedPath2d,
}

#[async_trait(?Send)]
impl CanvasOp for CanvasPathElem {
    fn prepare(
        &self,
        ts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + 'static> {
        let _ = ts;
        None::<EmptyFuture>
    }

    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice) {
        let _guard = CanvasStateGuard::new(canvas);

        if !set_transform(canvas, ts) {
            return;
        }
        // map_err(map_err("CanvasRenderTask.BuildPath2d")

        let mut fill_rule = None;
        let mut stroke_width = 0.;

        for style in &self.path_data.styles {
            match style {
                PathStyle::Fill(_) | PathStyle::Stroke(_) => {}
                PathStyle::StrokeWidth(width) => {
                    canvas.set_line_width(width.0 as f64);
                    stroke_width = width.0;
                }
                PathStyle::StrokeLineCap(cap) => {
                    canvas.set_line_cap(cap);
                }
                PathStyle::StrokeLineJoin(join) => {
                    canvas.set_line_join(join);
                }
                PathStyle::StrokeMitterLimit(limit) => {
                    canvas.set_miter_limit(limit.0 as f64);
                }
                PathStyle::StrokeDashArray(array) => {
                    let dash_array = js_sys::Array::from_iter(
                        array.iter().map(|d| JsValue::from_f64(d.0 as f64)),
                    );
                    canvas.set_line_dash(&dash_array);
                }
                PathStyle::StrokeDashOffset(offset) => {
                    canvas.set_line_dash_offset(offset.0 as f64);
                }
                PathStyle::FillRule(rule) => {
                    fill_rule = match rule.as_ref() {
                        "nonzero" => Some(CanvasWindingRule::Nonzero),
                        "evenodd" => Some(CanvasWindingRule::Evenodd),
                        _ => None,
                    };
                }
            }
        }

        let path = self.path.get_or_init(&self.path_data.d);

        if let Some(fill) = &self.fill {
            if fill.fill_radial_path(canvas, ts, &path) {
                // Non-uniform radial gradients are drawn through a clipped
                // paint transform to match SVG gradientTransform semantics.
            } else if fill.fill_conic_path(canvas, ts, &path, true) {
                // Conic gradients fall back to segmented drawing when native
                // canvas conics cannot represent the paint transform.
            } else {
                fill.set_fill_style(canvas, ts);
                if let Some(rule) = fill_rule {
                    canvas.fill_with_path_2d_and_winding(&path, rule);
                } else {
                    canvas.fill_with_path_2d(&path);
                }
            }
        }

        if stroke_width.abs() > 1e-5 {
            if let Some(stroke) = &self.stroke {
                stroke.set_stroke_style(canvas, ts);
                canvas.stroke_with_path(&path);
            }
        }

        #[cfg(feature = "render_bbox")]
        {
            // realize bbox
            let bbox = self.bbox_at(ts);
            render_bbox(canvas, bbox, "blue");

            #[cfg(feature = "report_bbox")]
            web_sys::console::log_1(
                &format!("bbox_at path {:?} {:?} {:?}", self.path_data, ts, bbox).into(),
            );
        }
    }
}

/// An image element.
#[derive(Debug)]
pub struct CanvasImageElem {
    pub image_data: ImageItem,
}

impl CanvasImageElem {
    fn prepare_image(image: Arc<Image>) -> Option<impl core::future::Future<Output = ()>> {
        let image_elem = rasterize_image(image.clone()).unwrap().0;

        let loaded = image_elem.loaded.lock().unwrap();
        if loaded.is_some() {
            return None;
        }

        let image = image.clone();
        Some(async move {
            wasm_bindgen_futures::JsFuture::from(image_elem.elem)
                .await
                .unwrap();
        })
    }

    async fn draw_image(ts: sk::Transform, canvas: &dyn CanvasDevice, image_data: &ImageItem) {
        if !set_transform(canvas, ts) {
            return;
        }

        let image = &image_data.image;

        let image_elem = rasterize_image(image.clone()).unwrap().0;
        let elem = wasm_bindgen_futures::JsFuture::from(image_elem.elem)
            .await
            .unwrap();

        // resize image to fit the view
        let (w, h) = {
            let size = image_data.size;
            let view_width = size.x.0;
            let view_height = size.y.0;

            let aspect = (image.width() as f32) / (image.height() as f32);

            let w: f32 = view_width.max(aspect * view_height);
            let h: f32 = w / aspect;
            (w as f64, h as f64)
        };

        let state = CanvasStateGuard::new(canvas);
        if !set_transform(canvas, ts) {
            return;
        }

        match elem.dyn_into::<ImageBitmap>() {
            Ok(image_elem) => {
                canvas.draw_image_with_image_bitmap_and_dw_and_dh(&image_elem, 0., 0., w, h);
            }
            Err(elem) => {
                let img = elem.dyn_into::<OffscreenCanvas>().expect("OffscreenCanvas");
                canvas.draw_image_with_offscreen_canvas_and_dw_and_dh(&img, 0., 0., w, h);
            }
        }
        drop(state);
    }
}

#[async_trait(?Send)]
impl CanvasOp for CanvasImageElem {
    fn prepare(
        &self,
        _ts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + 'static> {
        Self::prepare_image(self.image_data.image.clone())
    }

    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice) {
        Self::draw_image(ts, canvas, &self.image_data).await
    }
}

/// A glyph element.
#[derive(Debug)]
pub struct CanvasGlyphElem {
    pub fill: CanvasPaint,
    pub upem: Scalar,
    pub glyph_data: Arc<FlatGlyphItem>,
    pub path: CachedPath2d,
}

#[async_trait(?Send)]
impl CanvasOp for CanvasGlyphElem {
    fn prepare(
        &self,
        ts: sk::Transform,
    ) -> Option<impl core::future::Future<Output = ()> + 'static> {
        let _ = ts;
        match self.glyph_data.as_ref() {
            FlatGlyphItem::Image(glyph) => {
                CanvasImageElem::prepare_image(glyph.image.image.clone())
            }
            FlatGlyphItem::Outline(..) | FlatGlyphItem::None => None,
        }
    }

    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice) {
        if ts.sx == 0. || ts.sy == 0. {
            return;
        }

        // web_sys::console::log_1(&format!("realize glyph {ts:?}").into());

        let _guard = CanvasStateGuard::new(canvas);
        match self.glyph_data.as_ref() {
            #[cfg(not(feature = "rasterize_glyph"))]
            FlatGlyphItem::Outline(path) => {
                if self.fill.is_unsupported() {
                    return;
                }

                let path = self.path.get_or_init(&path.d);
                if self.fill.fill_conic_path(canvas, ts, &path, false) {
                    return;
                }

                if !set_transform(canvas, ts) {
                    return;
                }
                self.fill.set_fill_style(canvas, ts);
                canvas.fill_with_path_2d(&path);
            }
            #[cfg(feature = "rasterize_glyph")]
            FlatGlyphItem::Outline(path) => {
                if self.fill.is_unsupported() {
                    return;
                }

                let path_2d = self.path.get_or_init(&path.d);
                if self.fill.fill_conic_path(canvas, ts, &path_2d, false) {
                    return;
                }

                if self.fill.as_solid_str().is_none() {
                    if !set_transform(canvas, ts) {
                        return;
                    }
                    self.fill.set_fill_style(canvas, ts);
                    canvas.fill_with_path_2d(&path_2d);
                    return;
                }

                if ts.sx.abs() > 100. || ts.sy.abs() > 100. || ts.kx != 0. || ts.ky != 0. {
                    if !set_transform(canvas, ts) {
                        return;
                    }
                    self.fill.set_fill_style(canvas, ts);
                    canvas.fill_with_path_2d(&path_2d);
                    return;
                }

                let x = ts.tx;
                let y = ts.ty;

                let g = crate::pixglyph_canvas::Glyph::new(&path.d);

                let floor_x = x.floor() as i32;
                let floor_y = y.floor() as i32;
                let dx = x - floor_x as f32;
                let dy = y - floor_y as f32;

                let t = g.rasterize(dx, dy, ts.sx, ts.sy);

                crate::pixglyph_canvas::blend_glyph(
                    canvas,
                    &t,
                    self.fill.as_solid_str().unwrap(),
                    floor_x,
                    floor_y,
                );
            }
            FlatGlyphItem::Image(glyph) => {
                if !set_transform(canvas, ts) {
                    return;
                }
                CanvasImageElem::draw_image(ts.pre_concat(glyph.ts.into()), canvas, &glyph.image)
                    .await
            }
            FlatGlyphItem::None => {}
        }
    }
}

#[cfg(feature = "render_bbox")]
fn render_bbox(canvas: &dyn CanvasDevice, bbox: Option<Rect>, color: &str) {
    let Some(bbox) = bbox else {
        return;
    };

    let _guard = CanvasStateGuard::new(canvas);
    if !set_transform(canvas, sk::Transform::identity()) {
        return;
    }
    canvas.set_line_width(2.);
    canvas.set_stroke_style(&color.into());
    canvas.stroke_rect(
        bbox.lo.x.0 as f64,
        bbox.lo.y.0 as f64,
        bbox.width().0 as f64,
        bbox.height().0 as f64,
    );
}
