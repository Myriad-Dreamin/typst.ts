// todo
#![allow(clippy::arc_with_non_send_sync)]

mod bounds;
#[cfg(feature = "incremental")]
mod incr;
mod ops;
mod utils;

pub use bounds::BBoxAt;
use bounds::*;
pub use ops::*;

use ecow::EcoVec;
#[cfg(feature = "incremental")]
pub use incr::*;

use std::{cell::OnceCell, fmt::Debug, sync::Arc};

use tiny_skia as sk;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlDivElement, HtmlImageElement, Path2d};

use reflexo::{
    hash::Fingerprint,
    vector::{
        ir::{
            self, Abs, Axes, FlatGlyphItem, FontIndice, FontItem, FontRef, Image, ImmutStr, Module,
            Point, Ratio, Rect, Scalar, Size,
        },
        vm::{GroupContext, RenderVm, TransformContext},
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

            should_render_text_element: true,
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
    pub should_render_text_element: bool,
    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,

    _feat_phantom: std::marker::PhantomData<&'t Feat>,
}

impl<'m, 't, Feat: ExportFeature> FontIndice<'m> for CanvasRenderTask<'m, 't, Feat> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize)
    }
}

impl<'m, 't, Feat: ExportFeature> GlyphFactory for CanvasRenderTask<'m, 't, Feat> {
    fn get_glyph(&mut self, font: &FontItem, glyph: u32, fill: ImmutStr) -> Option<CanvasNode> {
        let glyph_data = font.get_glyph(glyph)?;
        Some(Arc::new(CanvasElem::Glyph(CanvasGlyphElem {
            fill,
            glyph_data: glyph_data.clone(),
        })))
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

            let w = text.width();

            CanvasBBox::Static(Box::new(Rect {
                lo: Point::new(Scalar(0.), accender - upem),
                hi: Point::new(w * upem / text.shape.size, accender),
            }))
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

    fn render_glyph(&mut self, ctx: &mut C, pos: Scalar, font: &FontItem, glyph: u32) {
        if let Some(glyph) = ctx.get_glyph(font, glyph, self.fill.clone().unwrap()) {
            self.inner.push((ir::Point::new(pos, Scalar(0.)), glyph));
        }
    }
}

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
        let elem = elem.0;

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

#[derive(Debug, Clone)]
struct UnsafeMemorize<T>(T);

unsafe impl<T> Send for UnsafeMemorize<T> {}
unsafe impl<T> Sync for UnsafeMemorize<T> {}

#[comemo::memoize]
fn rasterize_text(_fg: Fingerprint) -> Option<UnsafeMemorize<HtmlDivElement>> {
    let doc = web_sys::window()?.document()?;
    doc.create_element("div")
        .ok()?
        .dyn_into()
        .ok()
        .map(UnsafeMemorize)
}

fn create_image() -> Option<HtmlImageElement> {
    let doc = web_sys::window()?.document()?;
    doc.create_element("img").ok()?.dyn_into().ok()
}

#[comemo::memoize]
fn rasterize_image(_image: Arc<Image>) -> Option<UnsafeMemorize<HtmlImageElement>> {
    create_image().map(UnsafeMemorize)
}
