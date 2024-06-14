use async_trait::async_trait;

use crate::utils::EmptyFuture;
use ecow::EcoVec;

use std::{fmt::Debug, pin::Pin, sync::Arc};

use js_sys::Promise;
use tiny_skia as sk;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{HtmlImageElement, Path2d};

use reflexo::vector::ir::{self, FlatGlyphItem, Image, ImageItem, ImmutStr, PathStyle};

use super::{rasterize_image, set_transform, BBoxAt, CanvasBBox, CanvasStateGuard};

/// A reference to a canvas element.
pub type CanvasNode = Arc<CanvasElem>;

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

    async fn realize(&self, rts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let ts = rts.pre_concat(*self.ts.as_ref());

        for (pos, sub_elem) in &self.inner {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);
            sub_elem.realize(ts, canvas).await;
        }

        let _ = self.rect;
        let _ = Self::bbox_at;
        // #[cfg(feature = "report_group")]
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
    pub path_data: Box<ir::PathItem>,
    pub rect: CanvasBBox,
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
        let image_elem = rasterize_image(image.clone()).unwrap().0;

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

        let image_elem = rasterize_image(image.clone()).unwrap().0;
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
