use js_sys::Promise;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use tiny_skia as sk;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement, Path2d};

use typst_ts_core::{
    error::prelude::*,
    font::GlyphProvider,
    hash::Fingerprint,
    vector::{
        bbox::GlyphIndice,
        flat_ir::{
            self, FlatModule, LayoutRegionNode, LayoutSourceMapping, Module, ModuleBuilder,
            ModuleMetadata, MultiSvgDocument, Page, SourceMappingNode,
        },
        flat_vm::{FlatGroupContext, FlatRenderVm},
        ir::{
            self, Abs, Axes, BuildGlyph, FontIndice, FontRef, GlyphItem, GlyphPackBuilder,
            GlyphRef, Image, ImageItem, ImmutStr, PathStyle, Ratio, Rect, Scalar, Size, SvgItem,
        },
        vm::{GroupContext, RenderVm, TransformContext},
    },
    TakeAs,
};

mod content;
pub use content::TextContentTask;

mod annotation;
pub use annotation::AnnotationListTask;

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
pub type DefaultSvgTask = CanvasTask<DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_TRACING: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
}

use async_trait::async_trait;
#[async_trait(?Send)]
pub trait CanvasElem: Debug {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d);
}

pub type CanvasNode = Arc<Box<dyn CanvasElem + Send + Sync>>;

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

#[derive(Debug)]
pub struct CanvasGroupElem {
    pub ts: sk::Transform,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasGroupElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let ts = ts.pre_concat(self.ts);
        for (pos, sub_elem) in &self.inner {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);
            sub_elem.realize(ts, canvas).await;
        }
    }
}

#[derive(Debug)]
pub struct CanvasClipElem {
    pub ts: sk::Transform,
    pub d: ImmutStr,
    pub inner: CanvasNode,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasClipElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let _guard = CanvasStateGuard::new(canvas);

        set_transform(canvas, ts);
        canvas.clip_with_path_2d(&Path2d::new_with_path_string(&self.d).unwrap());

        self.inner.realize(ts, canvas).await
    }
}

#[derive(Debug)]
pub struct CanvasPathElem {
    pub path_data: ir::PathItem,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasPathElem {
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
            canvas.set_fill_style(&fill_color.as_ref().into());
            canvas.fill_with_path_2d(&Path2d::new_with_path_string(&self.path_data.d).unwrap());
        }

        if stroke && stroke_width.abs() > 1e-5 {
            canvas.set_stroke_style(&stroke_color.as_ref().into());
            canvas.stroke_with_path(&Path2d::new_with_path_string(&self.path_data.d).unwrap());
        }
    }
}

#[derive(Debug)]
pub struct CanvasImageElem {
    pub image_data: ImageItem,
}

impl CanvasImageElem {
    async fn load_image_cached(image: &Image, image_elem: &HtmlImageElement) {
        let image_loaded = image_elem.get_attribute("data-typst-loaded-image");
        match image_loaded {
            Some(t) if t == "true" => {}
            _ => {
                Self::load_image_slow(image, image_elem).await;
                image_elem
                    .set_attribute("data-typst-loaded-image", "true")
                    .unwrap();
            }
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

    async fn draw_image(
        ts: sk::Transform,
        canvas: &web_sys::CanvasRenderingContext2d,
        image_data: &ImageItem,
    ) {
        set_transform(canvas, ts);

        let image = &image_data.image;

        let image_elem = rasterize_image(image).unwrap();
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
impl CanvasElem for CanvasImageElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        Self::draw_image(ts, canvas, &self.image_data).await
    }
}

#[derive(Debug)]
pub struct CanvasGlyphElem {
    pub fill: ImmutStr,
    pub glyph_data: GlyphItem,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasGlyphElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let _guard = CanvasStateGuard::new(canvas);
        set_transform(canvas, ts);
        match &self.glyph_data {
            GlyphItem::Raw(..) => unreachable!(),
            GlyphItem::Outline(path) => {
                let fill: &str = &self.fill;
                canvas.set_fill_style(&fill.into());
                canvas.fill_with_path_2d(&Path2d::new_with_path_string(&path.d).unwrap());
            }
            GlyphItem::Image(glyph) => {
                CanvasImageElem::draw_image(ts.pre_concat(glyph.ts.into()), canvas, &glyph.image)
                    .await
            }
            GlyphItem::None => {}
        }
    }
}

/// Rework canvas render task with SVG's vector IR
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of SVG task.
pub struct CanvasRenderTask<'m, 't, Feat: ExportFeature> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    pub glyph_provider: GlyphProvider,

    pub module: &'m Module,

    /// Stores the glyphs used in the document.
    pub(crate) glyph_defs: &'t mut GlyphPackBuilder,

    /// See [`ExportFeature`].
    pub should_render_text_element: bool,
    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,

    pub _feat_phantom: std::marker::PhantomData<Feat>,
}

/// A builder for [`CanvasNode`].
/// It holds a reference to [`CanvasRenderTask`] and state of the building
/// process.
pub struct CanvasStack {
    pub ts: sk::Transform,
    pub clipper: Option<ir::PathItem>,
    pub fill: Option<ImmutStr>,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

impl From<CanvasStack> for CanvasNode {
    fn from(s: CanvasStack) -> Self {
        let inner: CanvasNode = Arc::new(Box::new(CanvasGroupElem {
            ts: s.ts,
            inner: s.inner,
        }));
        if let Some(clipper) = s.clipper {
            Arc::new(Box::new(CanvasClipElem {
                ts: s.ts,
                d: clipper.d.clone(),
                inner,
            }))
        } else {
            inner
        }
    }
}

/// Internal methods for [`CanvasStack`].
impl CanvasStack {
    pub fn with_text_shape(&mut self, shape: &ir::TextShape) {
        self.fill = Some(shape.fill.clone())
    }

    pub fn render_glyph_ref_inner(
        &mut self,
        pos: Scalar,
        _glyph_ref: &GlyphRef,
        glyph_data: &GlyphItem,
    ) {
        self.inner.push((
            ir::Point::new(pos, Scalar(0.)),
            Arc::new(Box::new(CanvasGlyphElem {
                fill: self.fill.clone().unwrap(),
                // todo: arc glyph item
                glyph_data: glyph_data.clone(),
            })),
        ))
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
impl<'m, C: BuildGlyph + RenderVm<Resultant = CanvasNode> + GlyphIndice<'m>> GroupContext<C>
    for CanvasStack
{
    fn render_item_at(&mut self, ctx: &mut C, pos: ir::Point, item: &ir::SvgItem) {
        self.inner.push((pos, ctx.render_item(item)));
    }

    fn render_glyph(&mut self, ctx: &mut C, pos: Scalar, glyph: &ir::GlyphItem) {
        let glyph_ref = ctx.build_glyph(glyph);
        if let Some(glyph_data) = ctx.get_glyph(&glyph_ref) {
            self.render_glyph_ref_inner(pos, &glyph_ref, glyph_data)
        }
    }

    fn render_path(&mut self, _ctx: &mut C, path: &ir::PathItem) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(Box::new(CanvasPathElem {
                path_data: path.clone(),
            })),
        ))
    }

    fn render_image(&mut self, _ctx: &mut C, image_item: &ir::ImageItem) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(Box::new(CanvasImageElem {
                image_data: image_item.clone(),
            })),
        ))
    }
}

/// See [`FlatGroupContext`].
impl<'m, C: FlatRenderVm<'m, Resultant = CanvasNode> + GlyphIndice<'m>> FlatGroupContext<C>
    for CanvasStack
{
    fn render_item_ref_at(&mut self, ctx: &mut C, pos: crate::ir::Point, item: &Fingerprint) {
        self.inner.push((pos, ctx.render_flat_item(item)));
    }

    fn render_glyph_ref(&mut self, ctx: &mut C, pos: Scalar, glyph: &GlyphRef) {
        if let Some(glyph_data) = ctx.get_glyph(glyph) {
            self.render_glyph_ref_inner(pos, glyph, glyph_data)
        }
    }
}

impl<'m, 't, Feat: ExportFeature> BuildGlyph for CanvasRenderTask<'m, 't, Feat> {
    fn build_font(&mut self, font: &typst::font::Font) -> FontRef {
        self.glyph_defs.build_font(font)
    }

    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> GlyphRef {
        self.glyph_defs.build_glyph(glyph)
    }
}

impl<'m, 't, Feat: ExportFeature> FontIndice<'m> for CanvasRenderTask<'m, 't, Feat> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize)
    }
}

impl<'m, 't, Feat: ExportFeature> GlyphIndice<'m> for CanvasRenderTask<'m, 't, Feat> {
    fn get_glyph(&self, g: &GlyphRef) -> Option<&'m ir::GlyphItem> {
        self.module.glyphs.get(g.glyph_idx as usize).map(|v| &v.1)
    }
}

impl<'m, 't, Feat: ExportFeature> RenderVm for CanvasRenderTask<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = CanvasNode;
    type Group = CanvasStack;

    fn start_group(&mut self) -> Self::Group {
        Self::Group {
            ts: sk::Transform::identity(),
            clipper: None,
            fill: None,
            inner: vec![],
        }
    }
}

impl<'m, 't, Feat: ExportFeature> FlatRenderVm<'m> for CanvasRenderTask<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = CanvasNode;
    type Group = CanvasStack;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m flat_ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&mut self, _v: &Fingerprint) -> Self::Group {
        Self::Group {
            ts: sk::Transform::identity(),
            clipper: None,
            fill: None,
            inner: vec![],
        }
    }

    fn start_flat_text(
        &mut self,
        value: &Fingerprint,
        text: &flat_ir::FlatTextItem,
    ) -> Self::Group {
        let mut g = self.start_flat_group(value);
        g.with_text_shape(&text.shape);
        g
    }
}

/// The task context for exporting canvas.
/// It is also as a namespace for all the functions used in the task.
pub struct CanvasTask<Feat: ExportFeature> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    glyph_provider: GlyphProvider,

    /// Stores the glyphs used in the document.
    pub(crate) glyph_defs: GlyphPackBuilder,

    _feat_phantom: std::marker::PhantomData<Feat>,
}

/// Unfortunately, `Default` derive does not work for generic structs.
impl<Feat: ExportFeature> Default for CanvasTask<Feat> {
    fn default() -> Self {
        Self {
            glyph_provider: GlyphProvider::default(),

            glyph_defs: GlyphPackBuilder::default(),

            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> CanvasTask<Feat> {
    /// fork a render task with module.
    pub fn fork_canvas_render_task<'m, 't>(
        &'t mut self,
        module: &'m flat_ir::Module,
    ) -> CanvasRenderTask<'m, 't, Feat> {
        CanvasRenderTask::<Feat> {
            glyph_provider: self.glyph_provider.clone(),

            module,

            glyph_defs: &mut self.glyph_defs,

            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }
}

pub struct IncrementalCanvasExporter {
    pub pixel_per_pt: f32,
    pub pages: Vec<(Arc<Box<dyn CanvasElem + Send + Sync>>, Size)>,
}

impl Default for IncrementalCanvasExporter {
    fn default() -> Self {
        Self {
            pixel_per_pt: 3.,
            pages: vec![],
        }
    }
}

impl IncrementalCanvasExporter {
    pub fn interpret_changes(&mut self, module: &Module, pages: &[Page]) {
        // render the document
        let mut t = CanvasTask::<DefaultExportFeature>::default();

        let mut ct = t.fork_canvas_render_task(module);

        let pages = pages
            .iter()
            .map(|Page { content, size }| (ct.render_flat_item(content), *size))
            .collect();
        self.pages = pages;
    }

    pub async fn flush_page(&mut self, idx: usize, canvas: &web_sys::CanvasRenderingContext2d) {
        let pg = &self.pages[idx];
        let ts = sk::Transform::from_scale(self.pixel_per_pt, self.pixel_per_pt);
        let sumy = self
            .pages
            .iter()
            .take(idx)
            .map(|(_, size)| size.y.0)
            .sum::<f32>();
        let ts = ts.pre_translate(0., sumy);

        pg.0.realize(ts, canvas).await;
    }
}

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
pub struct IncrCanvasDocClient {
    /// Full information of the current document from server.
    pub doc: MultiSvgDocument,

    pub elements: IncrementalCanvasExporter,

    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    pub doc_view: Option<Vec<Page>>,

    /// Optional source mapping data.
    pub source_mapping_data: Vec<SourceMappingNode>,
    /// Optional page source mapping references.
    pub page_source_mappping: LayoutSourceMapping,

    /// Don't use this
    /// it is public to make Default happy
    pub mb: ModuleBuilder,
}

impl IncrCanvasDocClient {
    /// Merge the delta from server.
    pub fn merge_delta(&mut self, delta: FlatModule) {
        self.doc.merge_delta(&delta);
        for metadata in delta.metadata {
            match metadata {
                ModuleMetadata::SourceMappingData(data) => {
                    self.source_mapping_data = data;
                }
                ModuleMetadata::PageSourceMapping(data) => {
                    self.page_source_mappping = data.take();
                }
                _ => {}
            }
        }

        let layout = self.doc.layouts.unwrap_single();
        let pages = layout.pages(&self.doc.module);
        if let Some(pages) = pages {
            self.elements
                .interpret_changes(pages.module(), pages.pages());
        }
    }

    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        self.elements.pixel_per_pt = pixel_per_pt;
    }

    /// Render the document in the given window.
    pub async fn render_in_window(
        &mut self,
        canvas: &web_sys::CanvasRenderingContext2d,
        rect: Rect,
    ) {
        // prepare an empty page for the pages that are not rendered
        // todo: better solution?
        let empty_page = self.mb.build(SvgItem::Group(Default::default()));
        self.doc
            .module
            .items
            .extend(self.mb.items.iter().map(|(f, (_, v))| (*f, v.clone())));

        // get previous doc_view
        // it is exact state of the current DOM.
        let prev_doc_view = self.doc_view.take().unwrap_or_default();

        // render next doc_view
        // for pages that is not in the view, we use empty_page
        // otherwise, we keep document layout
        let mut page_off: f32 = 0.;
        let mut next_doc_view = vec![];
        if !self.doc.layouts.is_empty() {
            let t = &self.doc.layouts[0]; // todo: multiple layout
            let pages = match t {
                LayoutRegionNode::Pages(a) => {
                    let (_, pages) = a.deref();
                    pages
                }
                _ => todo!(),
            };
            for page in pages.iter() {
                page_off += page.size.y.0;
                if page_off < rect.lo.y.0 || page_off - page.size.y.0 > rect.hi.y.0 {
                    next_doc_view.push(Page {
                        content: empty_page,
                        size: page.size,
                    });
                    continue;
                }

                next_doc_view.push(page.clone());
            }
        }

        for (idx, y) in next_doc_view.iter().enumerate() {
            let x = prev_doc_view.get(idx);
            if x.is_none() || (x.unwrap() != y && y.content != empty_page) {
                self.elements.flush_page(idx, canvas).await;
            }
        }
    }

    /// Render the document in the given window.
    pub async fn render_page_in_window(
        &mut self,
        canvas: &web_sys::CanvasRenderingContext2d,
        idx: usize,
        _rect: Rect,
    ) -> ZResult<()> {
        if idx >= self.elements.pages.len() {
            Err(error_once!("Renderer.OutofPageRange", idx: idx))?;
        }

        self.elements.flush_page(idx, canvas).await;

        Ok(())
    }
}

#[comemo::memoize]
fn rasterize_image(_image: &Image) -> Option<HtmlImageElement> {
    let window = web_sys::window().unwrap();
    window
        .document()
        .unwrap()
        .create_element("img")
        .unwrap()
        .dyn_into::<HtmlImageElement>()
        .ok()
}

// pub use backend::canvas::IncrCanvasDocClient;
