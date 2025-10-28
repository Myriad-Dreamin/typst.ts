// todo
#![allow(clippy::arc_with_non_send_sync)]

mod bounds;
mod device;
#[cfg(feature = "incremental")]
mod incr;
mod ops;
#[cfg(feature = "rasterize_glyph")]
mod pixglyph_canvas;
mod utils;

pub use bounds::BBoxAt;
pub use device::CanvasDevice;
#[cfg(feature = "incremental")]
pub use incr::*;
use js_sys::Promise;
pub use ops::*;
use web_sys::{Blob, HtmlImageElement, OffscreenCanvas, OffscreenCanvasRenderingContext2d};

use std::{
    cell::OnceCell,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use ecow::EcoVec;
use reflexo::{
    hash::Fingerprint,
    vector::{
        ir::{
            self, Abs, Axes, FontIndice, FontItem, FontRef, Image, ImmutStr, Module, Point, Ratio,
            Rect, Scalar, Size,
        },
        vm::{GroupContext, RenderVm, TransformContext},
    },
};
use tiny_skia as sk;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use bounds::*;

/// All the features that can be enabled or disabled.
pub trait ExportFeature {
    /// Whether to enable tracing.
    const ENABLE_TRACING: bool;
}

/// The default feature set which is used for exporting full-fledged canvas.
pub struct DefaultExportFeature;
/// The default feature set which is used for exporting canvas for printing.
pub type DefaultCanvasTask = CanvasTask<DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_TRACING: bool = false;
}

#[derive(Clone, Copy)]
pub struct BrowserFontMetric {
    pub semi_char_width: f32,
    pub full_char_width: f32,
    pub emoji_width: f32,
    // height: f32,
}

impl BrowserFontMetric {
    pub fn from_env() -> Self {
        let v = OffscreenCanvas::new(0, 0).expect("offscreen canvas is not supported");
        let ctx = v
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .unwrap();

        let _g = CanvasStateGuard::new(&ctx);
        ctx.set_font("128px monospace");
        let metrics = ctx.measure_text("A").unwrap();
        let semi_char_width = metrics.width();
        let metrics = ctx.measure_text("喵").unwrap();
        let full_char_width = metrics.width();
        let metrics = ctx.measure_text("🦄").unwrap();
        let emoji_width = metrics.width();
        // let a_height =
        //     (metrics.font_bounding_box_descent() +
        // metrics.font_bounding_box_ascent()).abs();

        Self {
            semi_char_width: (semi_char_width / 128.) as f32,
            full_char_width: (full_char_width / 128.) as f32,
            emoji_width: (emoji_width / 128.) as f32,
            // height: (a_height / 128.) as f32,
        }
    }

    /// Create a new instance for testing.
    /// The width are prime numbers for helping test.
    pub fn new_test() -> Self {
        Self {
            semi_char_width: 2.0,
            full_char_width: 3.0,
            emoji_width: 5.0,
            // height: 7.0,
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

            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }
}

trait GlyphFactory {
    fn get_glyph(&mut self, font: &FontItem, glyph: u32, fill: ImmutStr) -> Option<CanvasNode>;
}

/// Holds the data for rendering canvas.
///
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of SVG task.
pub struct CanvasRenderTask<'m, 't, Feat: ExportFeature> {
    /// The module which stores the frame data.
    pub module: &'m Module,

    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,

    _feat_phantom: std::marker::PhantomData<&'t Feat>,
}

impl<'m, Feat: ExportFeature> FontIndice<'m> for CanvasRenderTask<'m, '_, Feat> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize)
    }
}

impl<Feat: ExportFeature> GlyphFactory for CanvasRenderTask<'_, '_, Feat> {
    fn get_glyph(&mut self, font: &FontItem, glyph: u32, fill: ImmutStr) -> Option<CanvasNode> {
        let glyph_data = font.get_glyph(glyph)?;
        Some(Arc::new(CanvasElem::Glyph(CanvasGlyphElem {
            fill,
            upem: font.units_per_em,
            glyph_data: glyph_data.clone(),
        })))
    }
}

impl<'m, Feat: ExportFeature> RenderVm<'m> for CanvasRenderTask<'m, '_, Feat> {
    // type Resultant = String;
    type Resultant = CanvasNode;
    type Group = CanvasStack;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m ir::VecItem> {
        self.module.get_item(value)
    }

    fn start_group(&mut self, _v: &Fingerprint) -> Self::Group {
        Self::Group {
            kind: GroupKind::General,
            ts: sk::Transform::identity(),
            clipper: None,
            fill: None,
            inner: EcoVec::new(),
            rect: CanvasBBox::Dynamic(Box::new(OnceCell::new())),
        }
    }

    fn start_text(&mut self, value: &Fingerprint, text: &ir::TextItem) -> Self::Group {
        let mut g = self.start_group(value);
        g.kind = GroupKind::Text;
        g.rect = {
            // upem is the unit per em defined in the font.
            let font = self.get_font(&text.shape.font).unwrap();
            let upem = Scalar(font.units_per_em.0);
            let accender = Scalar(font.ascender.0) * upem;

            // todo: glyphs like macron has zero width... why?
            let w = text.width();

            if text.shape.size.0 == 0. {
                CanvasBBox::Static(Box::new(Rect {
                    lo: Point::new(Scalar(0.), accender - upem),
                    hi: Point::new(Scalar(0.), accender),
                }))
            } else {
                CanvasBBox::Static(Box::new(Rect {
                    lo: Point::new(Scalar(0.), accender - upem),
                    hi: Point::new(w * upem / text.shape.size, accender),
                }))
            }
        };
        for style in &text.shape.styles {
            if let ir::PathStyle::Fill(fill) = style {
                g.fill = Some(fill.clone());
            }
        }
        g
    }
}

/// A stacked builder for [`CanvasNode`].
///
/// It holds state of the building process.
pub struct CanvasStack {
    /// The kind of the group.
    pub kind: GroupKind,
    /// The transform matrix.
    pub ts: sk::Transform,
    /// A unique clip path on stack
    pub clipper: Option<ir::PathItem>,
    /// The fill color.
    pub fill: Option<ImmutStr>,
    /// The inner elements.
    pub inner: EcoVec<(ir::Point, CanvasNode)>,
    /// The bounding box of the group.
    pub rect: CanvasBBox,
}

impl From<CanvasStack> for CanvasNode {
    fn from(s: CanvasStack) -> Self {
        let inner: CanvasNode = Arc::new(CanvasElem::Group(CanvasGroupElem {
            ts: Box::new(s.ts),
            inner: s.inner,
            kind: s.kind,
            rect: s.rect,
        }));
        if let Some(clipper) = s.clipper {
            Arc::new(CanvasElem::Clip(CanvasClipElem {
                d: clipper.d,
                inner,
                clip_bbox: CanvasBBox::Dynamic(Box::new(OnceCell::new())),
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
impl<'m, C: RenderVm<'m, Resultant = CanvasNode> + GlyphFactory> GroupContext<C> for CanvasStack {
    fn render_path(&mut self, _ctx: &mut C, path: &ir::PathItem, _abs_ref: &Fingerprint) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(CanvasElem::Path(CanvasPathElem {
                path_data: Box::new(path.clone()),
                rect: CanvasBBox::Dynamic(Box::new(OnceCell::new())),
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

    fn render_item_at(&mut self, ctx: &mut C, pos: crate::ir::Point, item: &Fingerprint) {
        self.inner.push((pos, ctx.render_item(item)));
    }

    fn render_glyph(&mut self, ctx: &mut C, pos: Axes<Scalar>, font: &FontItem, glyph: u32) {
        if let Some(glyph) = ctx.get_glyph(font, glyph, self.fill.clone().unwrap()) {
            self.inner.push((ir::Point::new(pos.x, pos.y), glyph));
        }
    }
}

#[inline]
#[must_use]
fn set_transform(canvas: &dyn CanvasDevice, transform: sk::Transform) -> bool {
    if transform.sx == 0. || transform.sy == 0. {
        return false;
    }

    // see sync_transform
    let a = transform.sx as f64;
    let b = transform.ky as f64;
    let c = transform.kx as f64;
    let d = transform.sy as f64;
    let e = transform.tx as f64;
    let f = transform.ty as f64;

    canvas.set_transform(a, b, c, d, e, f);
    true
}

/// A guard for saving and restoring the canvas state.
///
/// When the guard is created, a cheap checkpoint of the canvas state is saved.
/// When the guard is dropped, the canvas state is restored.
pub struct CanvasStateGuard<'a>(&'a dyn CanvasDevice);

impl<'a> CanvasStateGuard<'a> {
    pub fn new(context: &'a dyn CanvasDevice) -> Self {
        context.save();
        Self(context)
    }
}

impl Drop for CanvasStateGuard<'_> {
    fn drop(&mut self) {
        self.0.restore();
    }
}

#[derive(Debug, Clone)]
struct UnsafeMemorize<T>(T);

// Safety: `UnsafeMemorize` is only used in wasm targets
unsafe impl<T> Send for UnsafeMemorize<T> {}
// Safety: `UnsafeMemorize` is only used in wasm targets
unsafe impl<T> Sync for UnsafeMemorize<T> {}

#[derive(Debug, Clone)]
struct LazyImage {
    elem: Promise,
    loaded: Arc<Mutex<Option<JsValue>>>,
}

fn create_image(image: Arc<Image>) -> Option<LazyImage> {
    let is_svg = image.format.contains("svg");

    web_sys::console::log_1(&format!("image format: {:?}", image.format).into());

    let u = js_sys::Uint8Array::new_with_length(image.data.len() as u32);
    u.copy_from(&image.data);

    let f = format!("image/{}", image.format);

    let blob = || {
        let parts = js_sys::Array::new();
        parts.push(&u);

        let tag = web_sys::BlobPropertyBag::new();
        tag.set_type(&f);
        web_sys::Blob::new_with_u8_array_sequence_and_options(
            &parts,
            // todo: security check
            // https://security.stackexchange.com/questions/148507/how-to-prevent-xss-in-svg-file-upload
            // todo: use our custom font
            &tag,
        )
        .unwrap()
    };

    let res = match web_sys::window() {
        Some(e) => {
            if is_svg {
                let blob = blob();
                Some(wasm_bindgen_futures::future_to_promise(async move {
                    // todo: image-rendering is not respected
                    let img = HtmlImageElement::new().unwrap();
                    let p = exception_create_image_blob(&blob, &img);
                    p.await;
                    Ok(html_image_to_bitmap(&img).into())
                }))
            } else {
                e.create_image_bitmap_with_blob(&blob()).ok()
            }
        }
        None => {
            let this = js_sys::global()
                .dyn_into::<web_sys::WorkerGlobalScope>()
                .unwrap();
            if is_svg {
                js_sys::Reflect::get(&this, &JsValue::from_str("loadSvg"))
                    .unwrap()
                    .dyn_into::<js_sys::Function>()
                    .unwrap()
                    .call2(&JsValue::NULL, &u, &f.into())
                    .unwrap()
                    .dyn_into::<Promise>()
                    .ok()
            } else {
                this.create_image_bitmap_with_blob(&blob()).ok()
            }
        }
    };

    let loaded = Arc::new(Mutex::new(None));

    let elem = res.map(|elem| {
        let loaded_that = loaded.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let elem = wasm_bindgen_futures::JsFuture::from(elem).await?;
            *loaded_that.lock().unwrap() = Some(elem.clone());
            Ok(elem)
        })
    });

    elem.map(|elem| LazyImage { elem, loaded })
}

pub fn html_image_to_bitmap(img: &HtmlImageElement) -> web_sys::ImageBitmap {
    let canvas = web_sys::OffscreenCanvas::new(img.width(), img.height()).unwrap();

    let ctx = canvas
        .get_context("2d")
        .expect("get context 2d")
        .expect("get context 2d");
    let ctx = ctx
        .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
        .expect("must be OffscreenCanvasRenderingContext2d");
    ctx.draw_image_with_html_image_element(img, 0., 0.)
        .expect("must draw_image_with_html_image_element");

    canvas
        .transfer_to_image_bitmap()
        .expect("transfer_to_image_bitmap")
}

pub async fn exception_create_image_blob(blob: &Blob, image_elem: &HtmlImageElement) {
    let data_url = web_sys::Url::create_object_url_with_blob(blob).unwrap();

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

#[comemo::memoize]
fn rasterize_image(e: Arc<Image>) -> Option<UnsafeMemorize<LazyImage>> {
    create_image(e).map(UnsafeMemorize)
}
