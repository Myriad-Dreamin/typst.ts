mod annotation;
mod content;
#[cfg(feature = "incremental")]
mod incr;
mod utils;

pub use annotation::AnnotationListTask;
pub use content::TextContentTask;
#[cfg(feature = "incremental")]
pub use incr::*;
use utils::EmptyFuture;

use std::{fmt::Debug, pin::Pin, sync::Arc};

use js_sys::Promise;
use tiny_skia as sk;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlDivElement, HtmlImageElement, Path2d};

use reflexo::{
    hash::Fingerprint,
    vector::{
        ir::{
            self, Abs, Axes, FlatGlyphItem, FontIndice, FontItem, FontRef, Image, ImageItem,
            ImmutStr, Module, PathStyle, Ratio, Scalar, Size,
        },
        vm::{GroupContext, RenderState, RenderVm, TransformContext},
    },
};

/// All the features that can be enabled or disabled.
pub trait ExportFeature {
    /// Whether to enable tracing.
    const ENABLE_TRACING: bool;

    /// Whether to render text element.
    /// The text elements is selectable and searchable.
    const SHOULD_RENDER_TEXT_ELEMENT: bool;
}

/// The default feature set which is used for exporting full-fledged svg.
pub struct DefaultExportFeature;
/// The default feature set which is used for exporting svg for printing.
pub type DefaultSvgTask = CanvasTask<DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_TRACING: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
}

use async_trait::async_trait;

/// The trait for all the operations that can be performed on some canvas
/// element.
#[async_trait(?Send)]
pub trait CanvasOp {
    /// Prepares the resource (recursively) for the action.
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + Sized + 'static>;
    /// Realizes the action on the canvas.
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d);
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
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + Sized + 'static> {
        type DynFutureBox = Pin<Box<dyn core::future::Future<Output = ()>>>;

        match self {
            CanvasElem::Group(g) => g.prepare().map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Clip(g) => g.prepare().map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Path(g) => g.prepare().map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Image(g) => g.prepare().map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
            CanvasElem::Glyph(g) => g.prepare().map(|e| {
                let e: DynFutureBox = Box::pin(e);
                e
            }),
        }
    }

    async fn realize(&self, _ts: sk::Transform, _canvas: &web_sys::CanvasRenderingContext2d) {
        match self {
            CanvasElem::Group(g) => g.realize(_ts, _canvas).await,
            CanvasElem::Clip(g) => g.realize(_ts, _canvas).await,
            CanvasElem::Path(g) => g.realize(_ts, _canvas).await,
            CanvasElem::Image(g) => g.realize(_ts, _canvas).await,
            CanvasElem::Glyph(g) => g.realize(_ts, _canvas).await,
        }
    }
}

// async fn realize(&self, ts: sk::Transform, canvas:
// &web_sys::CanvasRenderingContext2d);

/// A reference to a canvas element.
pub type CanvasNode = Arc<CanvasElem>;

#[inline]
fn set_transform(canvas: &web_sys::CanvasRenderingContext2d, transform: sk::Transform) {
    // see sync_transform
    let a = transform.sx as f64;
    let b = transform.ky as f64;
    let c = transform.kx as f64;
    let d = transform.sy as f64;
    let e = transform.tx as f64;
    let f = transform.ty as f64;

    let maybe_err = canvas.set_transform(a, b, c, d, e, f);
    // .map_err(map_err("CanvasRenderTask.SetTransform"))
    maybe_err.unwrap();
}

/// A guard for saving and restoring the canvas state.
///
/// When the guard is created, a cheap checkpoint of the canvas state is saved.
/// When the guard is dropped, the canvas state is restored.
pub struct CanvasStateGuard<'a>(&'a CanvasRenderingContext2d);

impl<'a> CanvasStateGuard<'a> {
    pub fn new(context: &'a CanvasRenderingContext2d) -> Self {
        context.save();
        Self(context)
    }
}

impl<'a> Drop for CanvasStateGuard<'a> {
    fn drop(&mut self) {
        self.0.restore();
    }
}

/// A group of canvas elements.
#[derive(Debug)]
pub struct CanvasGroupElem {
    pub ts: sk::Transform,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

#[async_trait(?Send)]
impl CanvasOp for CanvasGroupElem {
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + Sized + 'static> {
        let mut v = Vec::default();

        for (_, sub_elem) in &self.inner {
            if let Some(f) = sub_elem.prepare() {
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

    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let ts = ts.pre_concat(self.ts);
        for (pos, sub_elem) in &self.inner {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);
            sub_elem.realize(ts, canvas).await;
        }
    }
}

/// A reference to a canvas element with a clip path.
#[derive(Debug)]
pub struct CanvasClipElem {
    pub ts: sk::Transform,
    pub d: ImmutStr,
    pub inner: CanvasNode,
}

impl CanvasClipElem {
    pub fn realize_with<'a>(
        &self,
        ts: sk::Transform,
        canvas: &'a web_sys::CanvasRenderingContext2d,
    ) -> CanvasStateGuard<'a> {
        let guard = CanvasStateGuard::new(canvas);

        set_transform(canvas, ts);
        canvas.clip_with_path_2d(&Path2d::new_with_path_string(&self.d).unwrap());

        guard
    }
}

#[async_trait(?Send)]
impl CanvasOp for CanvasClipElem {
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + Sized + 'static> {
        self.inner.prepare()
    }

    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let _guard = self.realize_with(ts, canvas);

        self.inner.realize(ts, canvas).await
    }
}

/// A path element.
#[derive(Debug)]
pub struct CanvasPathElem {
    pub path_data: ir::PathItem,
}

#[async_trait(?Send)]
impl CanvasOp for CanvasPathElem {
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + 'static> {
        None::<EmptyFuture>
    }

    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let _guard = CanvasStateGuard::new(canvas);
        set_transform(canvas, ts);
        // map_err(map_err("CanvasRenderTask.BuildPath2d")

        let mut fill_color = "none".into();
        let mut fill = false;
        let mut stroke_color = "none".into();
        let mut stroke = false;
        let mut stroke_width = 0.;

        for style in &self.path_data.styles {
            match style {
                PathStyle::Fill(color) => {
                    fill_color = color.clone();
                    fill = true;
                }
                PathStyle::Stroke(color) => {
                    stroke_color = color.clone();
                    stroke = true;
                }
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
                    canvas.set_line_dash(&dash_array).unwrap();
                }
                PathStyle::StrokeDashOffset(offset) => {
                    canvas.set_line_dash_offset(offset.0 as f64);
                }
            }
        }

        if fill {
            // todo: canvas gradient and pattern
            if fill_color.starts_with('@') {
                fill_color = "black".into()
            }
            canvas.set_fill_style(&fill_color.as_ref().into());
            canvas.fill_with_path_2d(&Path2d::new_with_path_string(&self.path_data.d).unwrap());
        }

        if stroke && stroke_width.abs() > 1e-5 {
            // todo: canvas gradient and pattern
            if stroke_color.starts_with('@') {
                stroke_color = "black".into()
            }

            canvas.set_stroke_style(&stroke_color.as_ref().into());
            canvas.stroke_with_path(&Path2d::new_with_path_string(&self.path_data.d).unwrap());
        }
    }
}

/// An image element.
#[derive(Debug)]
pub struct CanvasImageElem {
    pub image_data: ImageItem,
}

impl CanvasImageElem {
    fn is_image_cached(image_elem: &HtmlImageElement) -> bool {
        let image_loaded = image_elem.get_attribute("data-typst-loaded-image");
        matches!(image_loaded, Some(t) if t == "true")
    }

    async fn load_image_cached(image: &Image, image_elem: &HtmlImageElement) {
        if !Self::is_image_cached(image_elem) {
            Self::load_image_slow(image, image_elem).await;
            image_elem
                .set_attribute("data-typst-loaded-image", "true")
                .unwrap();
        }
    }

    async fn load_image_slow(image: &Image, image_elem: &HtmlImageElement) {
        let u = js_sys::Uint8Array::new_with_length(image.data.len() as u32);
        u.copy_from(&image.data);

        let parts = js_sys::Array::new();
        parts.push(&u);
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &parts,
            // todo: security check
            // https://security.stackexchange.com/questions/148507/how-to-prevent-xss-in-svg-file-upload
            // todo: use our custom font
            web_sys::BlobPropertyBag::new().type_(&format!("image/{}", image.format)),
        )
        .unwrap();

        let data_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

        let img_load_promise = Promise::new(
            &mut move |complete: js_sys::Function, _reject: js_sys::Function| {
                let data_url = data_url.clone();
                let data_url2 = data_url.clone();
                let complete2 = complete.clone();

                image_elem.set_src(&data_url);

                // simulate async callback from another thread
                let a = Closure::<dyn Fn()>::new(move || {
                    web_sys::Url::revoke_object_url(&data_url).unwrap();
                    complete.call0(&complete).unwrap();
                });

                image_elem.set_onload(Some(a.as_ref().unchecked_ref()));
                a.forget();

                let a = Closure::<dyn Fn(JsValue)>::new(move |e: JsValue| {
                    web_sys::Url::revoke_object_url(&data_url2).unwrap();
                    complete2.call0(&complete2).unwrap();
                    // let end = std::time::Instant::now();
                    web_sys::console::log_1(
                        &format!(
                            "err image loading in {:?} {:?} {:?} {}",
                            // end - begin,
                            0,
                            js_sys::Reflect::get(&e, &"type".into()).unwrap(),
                            js_sys::JSON::stringify(&e).unwrap(),
                            data_url2,
                        )
                        .into(),
                    );
                });

                image_elem.set_onerror(Some(a.as_ref().unchecked_ref()));
                a.forget();
            },
        );

        wasm_bindgen_futures::JsFuture::from(img_load_promise)
            .await
            .unwrap();
    }

    fn prepare_image(image: Arc<Image>) -> Option<impl core::future::Future<Output = ()>> {
        let image_elem = rasterize_image(image.clone()).unwrap();

        if Self::is_image_cached(&image_elem) {
            return None;
        }

        let image = image.clone();
        Some(async move { Self::load_image_cached(&image, &image_elem).await })
    }

    async fn draw_image(
        ts: sk::Transform,
        canvas: &web_sys::CanvasRenderingContext2d,
        image_data: &ImageItem,
    ) {
        set_transform(canvas, ts);

        let image = &image_data.image;

        let image_elem = rasterize_image(image.clone()).unwrap();
        Self::load_image_cached(image, &image_elem).await;

        // resize image to fit the view
        let (w, h) = {
            let size = image_data.size;
            let view_width = size.x.0;
            let view_height = size.y.0;

            let aspect = (image.width() as f32) / (image.height() as f32);

            let w = view_width.max(aspect * view_height);
            let h = w / aspect;
            (w, h)
        };

        let state = CanvasStateGuard::new(canvas);
        set_transform(canvas, ts);
        canvas
            .draw_image_with_html_image_element_and_dw_and_dh(
                &image_elem,
                0.,
                0.,
                w as f64,
                h as f64,
            )
            .unwrap();
        drop(state);
    }
}

#[async_trait(?Send)]
impl CanvasOp for CanvasImageElem {
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + 'static> {
        Self::prepare_image(self.image_data.image.clone())
    }

    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        Self::draw_image(ts, canvas, &self.image_data).await
    }
}

/// A glyph element.
#[derive(Debug)]
pub struct CanvasGlyphElem {
    pub fill: ImmutStr,
    pub glyph_data: Arc<FlatGlyphItem>,
}

#[async_trait(?Send)]
impl CanvasOp for CanvasGlyphElem {
    fn prepare(&self) -> Option<impl core::future::Future<Output = ()> + 'static> {
        match self.glyph_data.as_ref() {
            FlatGlyphItem::Image(glyph) => {
                CanvasImageElem::prepare_image(glyph.image.image.clone())
            }
            FlatGlyphItem::Outline(..) | FlatGlyphItem::None => None,
        }
    }

    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let _guard = CanvasStateGuard::new(canvas);
        set_transform(canvas, ts);
        match self.glyph_data.as_ref() {
            FlatGlyphItem::Outline(path) => {
                let fill: &str = &self.fill;
                canvas.set_fill_style(&fill.into());
                canvas.fill_with_path_2d(&Path2d::new_with_path_string(&path.d).unwrap());
            }
            FlatGlyphItem::Image(glyph) => {
                CanvasImageElem::draw_image(ts.pre_concat(glyph.ts.into()), canvas, &glyph.image)
                    .await
            }
            FlatGlyphItem::None => {}
        }
    }
}

/// Holds the data for rendering canvas.
///
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of SVG task.
pub struct CanvasRenderTask<'m, 't, Feat: ExportFeature> {
    /// The module which stores the frame data.
    pub module: &'m Module,

    /// See [`ExportFeature`].
    pub should_render_text_element: bool,
    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,

    _feat_phantom: std::marker::PhantomData<&'t Feat>,
}

/// A stacked builder for [`CanvasNode`].
///
/// It holds state of the building process.
pub struct CanvasStack {
    /// The transform matrix.
    pub ts: sk::Transform,
    /// A unique clip path on stack
    pub clipper: Option<ir::PathItem>,
    /// The fill color.
    pub fill: Option<ImmutStr>,
    /// The inner elements.
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

impl From<CanvasStack> for CanvasNode {
    fn from(s: CanvasStack) -> Self {
        let inner: CanvasNode = Arc::new(CanvasElem::Group(CanvasGroupElem {
            ts: s.ts,
            inner: s.inner,
        }));
        if let Some(clipper) = s.clipper {
            Arc::new(CanvasElem::Clip(CanvasClipElem {
                ts: s.ts,
                d: clipper.d,
                inner,
            }))
        } else {
            inner
        }
    }
}

/// See [`TransformContext`].
impl<C> TransformContext<C> for CanvasStack {
    fn transform_matrix(mut self, _ctx: &mut C, m: &ir::Transform) -> Self {
        let sub_ts: sk::Transform = (*m).into();
        self.ts = self.ts.post_concat(sub_ts);
        self
    }

    fn transform_translate(mut self, _ctx: &mut C, matrix: Axes<Abs>) -> Self {
        self.ts = self.ts.post_translate(matrix.x.0, matrix.y.0);
        self
    }

    fn transform_scale(mut self, _ctx: &mut C, x: Ratio, y: Ratio) -> Self {
        self.ts = self.ts.post_scale(x.0, y.0);
        self
    }

    fn transform_rotate(self, _ctx: &mut C, _matrix: Scalar) -> Self {
        todo!()
    }

    fn transform_skew(mut self, _ctx: &mut C, matrix: (Ratio, Ratio)) -> Self {
        self.ts = self.ts.post_concat(sk::Transform {
            sx: 1.,
            sy: 1.,
            kx: matrix.0 .0,
            ky: matrix.1 .0,
            tx: 0.,
            ty: 0.,
        });
        self
    }

    fn transform_clip(mut self, _ctx: &mut C, matrix: &ir::PathItem) -> Self {
        self.clipper = Some(matrix.clone());
        self
    }
}

/// See [`GroupContext`].
impl<'m, C: RenderVm<'m, Resultant = CanvasNode>> GroupContext<C> for CanvasStack {
    fn render_path(
        &mut self,
        _state: RenderState,
        _ctx: &mut C,
        path: &ir::PathItem,
        _abs_ref: &Fingerprint,
    ) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(CanvasElem::Path(CanvasPathElem {
                path_data: path.clone(),
            })),
        ))
    }

    fn render_image(&mut self, _ctx: &mut C, image_item: &ir::ImageItem) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(CanvasElem::Image(CanvasImageElem {
                image_data: image_item.clone(),
            })),
        ))
    }

    fn render_item_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        pos: crate::ir::Point,
        item: &Fingerprint,
    ) {
        self.inner.push((pos, ctx.render_item(state, item)));
    }

    fn render_glyph(&mut self, _ctx: &mut C, pos: Scalar, font: &FontItem, glyph: u32) {
        if let Some(glyph_data) = font.get_glyph(glyph) {
            self.inner.push((
                ir::Point::new(pos, Scalar(0.)),
                Arc::new(CanvasElem::Glyph(CanvasGlyphElem {
                    fill: self.fill.clone().unwrap(),
                    glyph_data: glyph_data.clone(),
                })),
            ))
        }
    }
}

impl<'m, 't, Feat: ExportFeature> FontIndice<'m> for CanvasRenderTask<'m, 't, Feat> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize)
    }
}

impl<'m, 't, Feat: ExportFeature> RenderVm<'m> for CanvasRenderTask<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = CanvasNode;
    type Group = CanvasStack;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m ir::VecItem> {
        self.module.get_item(value)
    }

    fn start_group(&mut self, _v: &Fingerprint) -> Self::Group {
        Self::Group {
            ts: sk::Transform::identity(),
            clipper: None,
            fill: None,
            inner: vec![],
        }
    }

    fn start_text(
        &mut self,
        _state: RenderState,
        value: &Fingerprint,
        text: &ir::TextItem,
    ) -> Self::Group {
        let mut g = self.start_group(value);
        for style in &text.shape.styles {
            if let ir::PathStyle::Fill(fill) = style {
                g.fill = Some(fill.clone());
            }
        }
        g
    }
}

/// The task context for exporting canvas.
/// It is also as a namespace for all the functions used in the task.
pub struct CanvasTask<Feat: ExportFeature> {
    _feat_phantom: std::marker::PhantomData<Feat>,
}

/// Unfortunately, `Default` derive does not work for generic structs.
impl<Feat: ExportFeature> Default for CanvasTask<Feat> {
    fn default() -> Self {
        Self {
            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> CanvasTask<Feat> {
    /// fork a render task with module.
    pub fn fork_canvas_render_task<'m, 't>(
        &'t mut self,
        module: &'m ir::Module,
    ) -> CanvasRenderTask<'m, 't, Feat> {
        CanvasRenderTask::<Feat> {
            module,

            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }
}

/// A rendered page of canvas.
#[derive(Clone)]
pub struct CanvasPage {
    /// A rendered canvas element.
    pub elem: CanvasNode,
    /// The fingerprint of the content for identifying page difference.
    pub content: Fingerprint,
    /// The size of the page.
    pub size: Size,
}

/// Useful snippets for rendering parts of vector items to canvas.
pub struct CanvasRenderSnippets;

impl CanvasRenderSnippets {
    fn put_glyph(
        canvas: &web_sys::CanvasRenderingContext2d,
        fill: &str,
        glyph_item: &FlatGlyphItem,
        ts: sk::Transform,
    ) {
        let _guard = CanvasStateGuard::new(canvas);
        set_transform(canvas, ts);
        match &glyph_item {
            FlatGlyphItem::Outline(path) => {
                canvas.set_fill_style(&fill.into());
                canvas.fill_with_path_2d(&Path2d::new_with_path_string(&path.d).unwrap());
            }
            FlatGlyphItem::Image(_glyph) => {
                unimplemented!();
            }
            FlatGlyphItem::None => {}
        }
    }

    /// Rasterize a text element to a image based on canvas.
    pub fn rasterize_text<'a>(
        fg: &Fingerprint,
        glyphs: impl Iterator<Item = (Scalar, &'a FlatGlyphItem)>,
        width: f32,
        height: f32,
        decender: f32,
        fill: &str,
    ) -> String {
        let Some(elem) = rasterize_text(*fg) else {
            return Default::default();
        };
        let image_loaded = elem.get_attribute("data-typst-loaded-image");
        if matches!(image_loaded, Some(t) if t == "true") {
            return elem.outer_html();
        }

        let random_token = format!(
            "text-{}",
            js_sys::Math::random().to_string().replace('.', "")
        );

        // presentational text
        elem.set_class_name(format!("typst-ptext {}", random_token).as_str());
        elem.set_attribute("data-typst-loaded-image", "true")
            .unwrap();

        crate::utils::console_log!(
            "rasterize_text {:?} {} {} {} {}",
            fg,
            fill,
            width,
            height,
            decender
        );

        elem.set_attribute(
            "style",
            "width: 100%; height: 100%; background: transparent;",
        )
        .unwrap();

        Self::rasterize_text_slow(
            elem.clone(),
            random_token,
            glyphs,
            width,
            height,
            decender,
            fill,
        );

        elem.outer_html()
    }

    fn rasterize_text_slow<'a>(
        elem: HtmlDivElement,
        random_token: String,
        glyphs: impl Iterator<Item = (Scalar, &'a FlatGlyphItem)>,
        width: f32,
        height: f32,
        decender: f32,
        fill: &str,
    ) {
        const RATIO: f32 = 8f32;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas.set_width((width / RATIO).ceil() as u32);
        canvas.set_height(((height + decender) / RATIO).ceil() as u32);
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let ts = sk::Transform::from_scale(1. / RATIO, 1. / RATIO).pre_translate(0., decender);
        for (pos, glyph) in glyphs {
            Self::put_glyph(&ctx, fill, glyph, ts.pre_translate(pos.0, 0.));
        }

        // window.handleTextRasterized = function (canvas: HTMLCanvasElement, elem:
        // Element, randomToken: string) get handle and call
        let window = web_sys::window().unwrap();
        if let Ok(proc) = js_sys::Reflect::get(&window, &JsValue::from_str("handleTextRasterized"))
        {
            proc.dyn_ref::<js_sys::Function>()
                .unwrap()
                .call3(&JsValue::NULL, &canvas, &elem, &random_token.into())
                .unwrap();
        }
    }
}

// pub use backend::canvas::IncrCanvasDocClient;

fn create_image() -> Option<HtmlImageElement> {
    let doc = web_sys::window()?.document()?;
    doc.create_element("img").ok()?.dyn_into().ok()
}

#[comemo::memoize(local)]
fn rasterize_image(_image: Arc<Image>) -> Option<HtmlImageElement> {
    create_image()
}

#[comemo::memoize(local)]
fn rasterize_text(_fg: Fingerprint) -> Option<HtmlDivElement> {
    let doc = web_sys::window()?.document()?;
    doc.create_element("div").ok()?.dyn_into().ok()
}
