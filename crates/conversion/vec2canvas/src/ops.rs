#![allow(unused)]

use async_trait::async_trait;
use reflexo_vec2bbox::Vec2BBoxPass;

use crate::{utils::EmptyFuture, CanvasDevice, CanvasPaint};
use ecow::EcoVec;

use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashMap,
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
use web_sys::{
    CanvasWindingRule, ImageBitmap, OffscreenCanvas, OffscreenCanvasRenderingContext2d, Path2d,
};

use reflexo::vector::ir::{
    self, FlatGlyphItem, Image, ImageItem, ImmutStr, PathStyle, Rect, Scalar,
};

use super::{rasterize_image, set_transform, BBoxAt, CanvasBBox, CanvasStateGuard};

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
const BITMAP_CACHE_PADDING: f32 = 2.0;
#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
const BITMAP_CACHE_MAX_DIMENSION: f32 = 4096.0;
#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
const BITMAP_CACHE_MAX_AREA: f32 = 16_000_000.0;

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

#[derive(Debug, Clone, Copy, Default)]
pub struct CanvasRenderContext {
    window: Option<Rect>,
}

impl CanvasRenderContext {
    pub fn new(window: Option<Rect>) -> Self {
        Self { window }
    }

    fn render_window_intersects(self, node: &CanvasNode, ts: sk::Transform) -> bool {
        let Some(window) = self.window else {
            return true;
        };

        node.bbox_at(ts)
            .map(|bbox| rect_intersects(bbox, window))
            .unwrap_or(true)
    }
}

fn rect_intersects(a: Rect, b: Rect) -> bool {
    a.left().0 <= b.right().0
        && a.right().0 >= b.left().0
        && a.top().0 <= b.bottom().0
        && a.bottom().0 >= b.top().0
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
    async fn realize(&self, ts: sk::Transform, canvas: &dyn CanvasDevice, ctx: CanvasRenderContext);
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

    async fn realize(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) {
        match self {
            CanvasElem::Group(g) => g.realize(ts, canvas, ctx).await,
            CanvasElem::Clip(g) => g.realize(ts, canvas, ctx).await,
            CanvasElem::Path(g) => g.realize(ts, canvas, ctx).await,
            CanvasElem::Image(g) => g.realize(ts, canvas, ctx).await,
            CanvasElem::Glyph(g) => g.realize(ts, canvas, ctx).await,
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
    #[cfg(feature = "bitmap_cache_word")]
    pub bitmap_cache: RefCell<Option<GroupBitmapCache>>,
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

    async fn realize(
        &self,
        rts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) {
        let ts = rts.pre_concat(*self.ts.as_ref());

        #[cfg(feature = "bitmap_cache_word")]
        if bitmap_cache_enabled() && matches!(self.kind, GroupKind::Text) {
            if self.realize_group_bitmap(rts, ts, canvas, ctx).await {
                return;
            }
        }

        #[cfg(any(feature = "bitmap_cache_line", feature = "bitmap_cache_paragraph"))]
        if bitmap_cache_enabled() && matches!(self.kind, GroupKind::General) {
            self.realize_inner_with_text_bitmap_spans(ts, canvas, ctx)
                .await;
            self.realize_debug(rts, ts, canvas);
            return;
        }

        #[cfg(not(feature = "rasterize_glyph"))]
        if matches!(self.kind, GroupKind::Text) && self.realize_solid_text_run(ts, canvas, ctx) {
            self.realize_debug(rts, ts, canvas);
            return;
        }

        self.realize_inner(ts, canvas, ctx).await;
        self.realize_debug(rts, ts, canvas);
    }
}

impl CanvasGroupElem {
    #[cfg(not(feature = "rasterize_glyph"))]
    fn realize_solid_text_run(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) -> bool {
        let Some(fill) = self.solid_text_run_fill() else {
            return false;
        };

        let _guard = CanvasStateGuard::new(canvas);
        canvas.set_fill_style_str(fill);
        for (pos, sub_elem) in &self.inner {
            let sub_ts = ts.pre_translate(pos.x.0, pos.y.0);
            if !ctx.render_window_intersects(sub_elem, sub_ts) {
                continue;
            }

            let CanvasElem::Glyph(glyph) = sub_elem.as_ref() else {
                return false;
            };
            let FlatGlyphItem::Outline(path) = glyph.glyph_data.as_ref() else {
                continue;
            };
            if !set_transform(canvas, sub_ts) {
                continue;
            }

            let path = glyph.path.get_or_init(&path.d);
            canvas.fill_with_path_2d(path);
        }

        true
    }

    #[cfg(not(feature = "rasterize_glyph"))]
    fn solid_text_run_fill(&self) -> Option<&str> {
        let mut fill: Option<&str> = None;
        for (_, sub_elem) in &self.inner {
            let CanvasElem::Glyph(glyph) = sub_elem.as_ref() else {
                return None;
            };

            match glyph.glyph_data.as_ref() {
                FlatGlyphItem::Outline(_) | FlatGlyphItem::None => {}
                FlatGlyphItem::Image(_) => return None,
            }

            let glyph_fill = glyph.fill.as_solid_str()?;
            if let Some(fill) = fill {
                if fill != glyph_fill {
                    return None;
                }
            } else {
                fill = Some(glyph_fill);
            }
        }

        fill
    }

    async fn realize_inner(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) {
        for (pos, sub_elem) in &self.inner {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);
            if !ctx.render_window_intersects(sub_elem, ts) {
                continue;
            }
            sub_elem.realize(ts, canvas, ctx).await;
        }
    }

    fn realize_debug(&self, rts: sk::Transform, ts: sk::Transform, canvas: &dyn CanvasDevice) {
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

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
#[derive(Debug)]
pub struct GroupBitmapCache {
    key: BitmapCacheKey,
    bitmap: CachedBitmap,
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
#[derive(Debug)]
struct CachedBitmap {
    canvas: OffscreenCanvas,
    width: u32,
    height: u32,
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct BitmapCacheKey {
    sx: i32,
    ky: i32,
    kx: i32,
    sy: i32,
    tx: i32,
    ty: i32,
    width: u32,
    height: u32,
}

#[cfg(any(feature = "bitmap_cache_line", feature = "bitmap_cache_paragraph"))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct SpanBitmapCacheKey {
    group: usize,
    start: usize,
    end: usize,
    bitmap: BitmapCacheKey,
}

#[cfg(any(feature = "bitmap_cache_line", feature = "bitmap_cache_paragraph"))]
thread_local! {
    static SPAN_BITMAP_CACHE: RefCell<HashMap<SpanBitmapCacheKey, CachedBitmap>> =
        RefCell::new(HashMap::new());
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
thread_local! {
    static BITMAP_CACHE_DEPTH: Cell<u32> = const { Cell::new(0) };
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
struct BitmapCacheScope;

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
impl BitmapCacheScope {
    fn new() -> Self {
        BITMAP_CACHE_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Self
    }
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
impl Drop for BitmapCacheScope {
    fn drop(&mut self) {
        BITMAP_CACHE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
fn bitmap_cache_enabled() -> bool {
    BITMAP_CACHE_DEPTH.with(|depth| depth.get() == 0)
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
fn bitmap_cache_key(ts: sk::Transform, bbox: Rect, width: u32, height: u32) -> BitmapCacheKey {
    fn q(v: f32) -> i32 {
        (v * 1024.0).round() as i32
    }

    BitmapCacheKey {
        sx: q(ts.sx),
        ky: q(ts.ky),
        kx: q(ts.kx),
        sy: q(ts.sy),
        tx: q(ts.tx + bbox.left().0),
        ty: q(ts.ty + bbox.top().0),
        width,
        height,
    }
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
fn bitmap_dimensions(bbox: Rect) -> Option<(u32, u32)> {
    let width = bbox.width().0 + BITMAP_CACHE_PADDING * 2.0;
    let height = bbox.height().0 + BITMAP_CACHE_PADDING * 2.0;
    if width <= 0.0
        || height <= 0.0
        || width > BITMAP_CACHE_MAX_DIMENSION
        || height > BITMAP_CACHE_MAX_DIMENSION
        || width * height > BITMAP_CACHE_MAX_AREA
    {
        return None;
    }

    Some((width.ceil() as u32, height.ceil() as u32))
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
fn draw_cached_bitmap(canvas: &dyn CanvasDevice, bitmap: &CachedBitmap, bbox: Rect) {
    canvas.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    canvas.draw_image_with_offscreen_canvas(
        &bitmap.canvas,
        (bbox.left().0 - BITMAP_CACHE_PADDING) as f64,
        (bbox.top().0 - BITMAP_CACHE_PADDING) as f64,
    );
}

#[cfg(any(
    feature = "bitmap_cache_word",
    feature = "bitmap_cache_line",
    feature = "bitmap_cache_paragraph"
))]
fn create_bitmap_canvas(
    width: u32,
    height: u32,
) -> Option<(OffscreenCanvas, OffscreenCanvasRenderingContext2d)> {
    let canvas = OffscreenCanvas::new(width, height).ok()?;
    let context = canvas
        .get_context("2d")
        .ok()
        .flatten()?
        .dyn_into::<OffscreenCanvasRenderingContext2d>()
        .ok()?;
    Some((canvas, context))
}

#[cfg(feature = "bitmap_cache_word")]
impl CanvasGroupElem {
    async fn realize_group_bitmap(
        &self,
        rts: sk::Transform,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) -> bool {
        if ts.kx != 0.0 || ts.ky != 0.0 {
            return false;
        }

        let Some(bbox) = self.bbox_at(rts) else {
            return false;
        };
        let Some((width, height)) = bitmap_dimensions(bbox) else {
            return false;
        };
        let key = bitmap_cache_key(ts, bbox, width, height);

        {
            let cache = self.bitmap_cache.borrow();
            if let Some(cache) = cache.as_ref().filter(|cache| cache.key == key) {
                draw_cached_bitmap(canvas, &cache.bitmap, bbox);
                return true;
            }
        }

        let Some((bitmap_canvas, bitmap_context)) = create_bitmap_canvas(width, height) else {
            return false;
        };
        let bitmap_device: &dyn CanvasDevice = &bitmap_context;
        let shifted_ts = ts.post_translate(
            -bbox.left().0 + BITMAP_CACHE_PADDING,
            -bbox.top().0 + BITMAP_CACHE_PADDING,
        );

        {
            let _scope = BitmapCacheScope::new();
            self.realize_inner(shifted_ts, bitmap_device, ctx).await;
        }

        let bitmap = CachedBitmap {
            canvas: bitmap_canvas,
            width,
            height,
        };
        draw_cached_bitmap(canvas, &bitmap, bbox);
        *self.bitmap_cache.borrow_mut() = Some(GroupBitmapCache { key, bitmap });
        true
    }
}

#[cfg(any(feature = "bitmap_cache_line", feature = "bitmap_cache_paragraph"))]
impl CanvasGroupElem {
    async fn realize_inner_with_text_bitmap_spans(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) {
        let mut idx = 0;
        while idx < self.inner.len() {
            if !is_text_group(&self.inner[idx].1) {
                let (pos, sub_elem) = &self.inner[idx];
                let sub_ts = ts.pre_translate(pos.x.0, pos.y.0);
                if !ctx.render_window_intersects(sub_elem, sub_ts) {
                    idx += 1;
                    continue;
                }
                sub_elem.realize(sub_ts, canvas, ctx).await;
                idx += 1;
                continue;
            }

            let end = self.collect_text_bitmap_span(ts, idx);
            if end > idx
                && self
                    .realize_text_bitmap_span(ts, idx, end, canvas, ctx)
                    .await
            {
                idx = end;
                continue;
            }

            let (pos, sub_elem) = &self.inner[idx];
            let sub_ts = ts.pre_translate(pos.x.0, pos.y.0);
            if !ctx.render_window_intersects(sub_elem, sub_ts) {
                idx += 1;
                continue;
            }
            sub_elem.realize(sub_ts, canvas, ctx).await;
            idx += 1;
        }
    }

    fn collect_text_bitmap_span(&self, ts: sk::Transform, start: usize) -> usize {
        let Some(mut bbox) = self.span_bbox_at(ts, start, start + 1) else {
            return start + 1;
        };
        let mut end = start + 1;

        while end < self.inner.len() && is_text_group(&self.inner[end].1) {
            let Some(next_bbox) = self.span_bbox_at(ts, end, end + 1) else {
                break;
            };

            #[cfg(feature = "bitmap_cache_line")]
            if !same_text_line(bbox, next_bbox) {
                break;
            }

            #[cfg(feature = "bitmap_cache_paragraph")]
            if !same_text_paragraph(bbox, next_bbox) {
                break;
            }

            let candidate = bbox.union(&next_bbox);
            if bitmap_dimensions(candidate).is_none() {
                break;
            }

            bbox = candidate;
            end += 1;
        }

        end
    }

    fn span_bbox_at(&self, ts: sk::Transform, start: usize, end: usize) -> Option<Rect> {
        self.inner[start..end]
            .iter()
            .fold(None, |acc: Option<Rect>, (pos, elem)| {
                let bbox = elem.bbox_at(ts.pre_translate(pos.x.0, pos.y.0))?;
                Some(acc.map_or(bbox, |acc| acc.union(&bbox)))
            })
    }

    async fn realize_text_bitmap_span(
        &self,
        ts: sk::Transform,
        start: usize,
        end: usize,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) -> bool {
        if ts.kx != 0.0 || ts.ky != 0.0 {
            return false;
        }

        let Some(bbox) = self.span_bbox_at(ts, start, end) else {
            return false;
        };
        let Some((width, height)) = bitmap_dimensions(bbox) else {
            return false;
        };
        let key = SpanBitmapCacheKey {
            group: self as *const CanvasGroupElem as usize,
            start,
            end,
            bitmap: bitmap_cache_key(ts, bbox, width, height),
        };

        if SPAN_BITMAP_CACHE.with(|cache| {
            let cache = cache.borrow();
            if let Some(bitmap) = cache.get(&key) {
                draw_cached_bitmap(canvas, bitmap, bbox);
                true
            } else {
                false
            }
        }) {
            return true;
        }

        let Some((bitmap_canvas, bitmap_context)) = create_bitmap_canvas(width, height) else {
            return false;
        };
        let bitmap_device: &dyn CanvasDevice = &bitmap_context;
        let shifted_ts = ts.post_translate(
            -bbox.left().0 + BITMAP_CACHE_PADDING,
            -bbox.top().0 + BITMAP_CACHE_PADDING,
        );

        {
            let _scope = BitmapCacheScope::new();
            for (pos, sub_elem) in &self.inner[start..end] {
                sub_elem
                    .realize(
                        shifted_ts.pre_translate(pos.x.0, pos.y.0),
                        bitmap_device,
                        ctx,
                    )
                    .await;
            }
        }

        let bitmap = CachedBitmap {
            canvas: bitmap_canvas,
            width,
            height,
        };
        draw_cached_bitmap(canvas, &bitmap, bbox);
        SPAN_BITMAP_CACHE.with(|cache| {
            cache.borrow_mut().insert(key, bitmap);
        });
        true
    }
}

#[cfg(any(feature = "bitmap_cache_line", feature = "bitmap_cache_paragraph"))]
fn is_text_group(node: &CanvasNode) -> bool {
    matches!(
        node.as_ref(),
        CanvasElem::Group(group) if matches!(group.kind, GroupKind::Text)
    )
}

#[cfg(feature = "bitmap_cache_line")]
fn same_text_line(a: Rect, b: Rect) -> bool {
    let top = a.top().0.max(b.top().0);
    let bottom = a.bottom().0.min(b.bottom().0);
    let overlap = bottom - top;
    let min_height = a.height().0.min(b.height().0).max(1.0);
    overlap > min_height * 0.5
}

#[cfg(feature = "bitmap_cache_paragraph")]
fn same_text_paragraph(a: Rect, b: Rect) -> bool {
    let vertical_gap = if b.top().0 > a.bottom().0 {
        b.top().0 - a.bottom().0
    } else if a.top().0 > b.bottom().0 {
        a.top().0 - b.bottom().0
    } else {
        0.0
    };
    let line_height = a.height().0.max(b.height().0).max(1.0);
    vertical_gap < line_height * 1.5
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

    async fn realize(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        ctx: CanvasRenderContext,
    ) {
        let _guard = self.realize_with(ts, canvas);

        self.inner.realize(ts, canvas, ctx).await
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

    async fn realize(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        _ctx: CanvasRenderContext,
    ) {
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
            if fill.fill_radial_path(canvas, ts, path) {
                // Non-uniform radial gradients are drawn through a clipped
                // paint transform to match SVG gradientTransform semantics.
            } else if fill.fill_conic_path(canvas, ts, path, true) {
                // Conic gradients fall back to segmented drawing when native
                // canvas conics cannot represent the paint transform.
            } else {
                fill.set_fill_style(canvas, ts);
                if let Some(rule) = fill_rule {
                    canvas.fill_with_path_2d_and_winding(path, rule);
                } else {
                    canvas.fill_with_path_2d(path);
                }
            }
        }

        if stroke_width.abs() > 1e-5 {
            if let Some(stroke) = &self.stroke {
                stroke.set_stroke_style(canvas, ts);
                canvas.stroke_with_path(path);
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

    async fn realize(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        _ctx: CanvasRenderContext,
    ) {
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

    async fn realize(
        &self,
        ts: sk::Transform,
        canvas: &dyn CanvasDevice,
        _ctx: CanvasRenderContext,
    ) {
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
                if self.fill.fill_conic_path(canvas, ts, path, false) {
                    return;
                }

                if !set_transform(canvas, ts) {
                    return;
                }
                self.fill.set_fill_style(canvas, ts);
                canvas.fill_with_path_2d(path);
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
