#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::all)]

use std::{fmt::Debug, ops::Deref, sync::Arc};
use tiny_skia as sk;

use wasm_bindgen::JsValue;
use web_sys::Path2d;

use typst_ts_core::{
    font::GlyphProvider,
    hash::{Fingerprint, FingerprintBuilder},
    vector::{
        bbox::GlyphIndice,
        flat_ir::{
            FlatModule, LayoutRegionNode, LayoutSourceMapping, ModuleBuilder, ModuleMetadata,
            MultiSvgDocument, Page, SourceMappingNode,
        },
        flat_vm::{FlatGroupContext, FlatRenderVm},
        ir::{
            self, Abs, Axes, BuildGlyph, GlyphItem, GlyphPackBuilder, GlyphRef, ImageItem,
            ImmutStr, PathStyle, Ratio, Rect, Scalar, Size, SvgItem,
        },
        vm::{GroupContext, RenderVm, TransformContext},
    },
    TakeAs,
};

use crate::{flat_ir, DefaultExportFeature, SvgTask};
use crate::{ExportFeature, Module};

use async_trait::async_trait;
#[async_trait(?Send)]
pub trait CanvasElem: Debug {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d);
}

pub type CanvasNode = Arc<Box<dyn CanvasElem>>;

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

#[derive(Debug)]
pub struct CanvasGroupElem {
    pub ts: sk::Transform,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasGroupElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        web_sys::console::log_2(&"CanvasGroupElem".into(), &"realize".into());
        let ts = ts.post_concat(self.ts);
        for (pos, sub_elem) in &self.inner {
            let ts = ts.post_translate(pos.x.0, pos.y.0);
            sub_elem.realize(ts, canvas).await;
        }
    }
}

#[derive(Debug)]
pub struct CanvasPathElem {
    pub path_data: ir::PathItem,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasPathElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        set_transform(canvas, ts);
        // todo style
        // map_err(map_err("CanvasRenderTask.BuildPath2d")

        let mut fill_color = "none".into();
        let mut fill = false;
        let mut stroke_color = "none".into();
        let mut stroke = false;

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

        if stroke {
            canvas.set_stroke_style(&stroke_color.as_ref().into());
            canvas.stroke_with_path(&Path2d::new_with_path_string(&self.path_data.d).unwrap());
        }
    }
}

#[derive(Debug)]
pub struct CanvasImageElem {
    pub image_data: ImageItem,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasImageElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        set_transform(canvas, ts);
        // self.content.push(SvgText::Plain(render_image(
        //     &image_item.image,
        //     image_item.size,
        // )))

        // self.t.canvas.draw_image_with_html_image_element_and_dw_and_dh(, dx,
        // dy, dw, dh)
        // todo!()
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
        set_transform(canvas, ts);
        match &self.glyph_data {
            GlyphItem::Raw(..) => unreachable!(),
            GlyphItem::Outline(path) => {
                web_sys::console::log_2(&"CanvasGlyphElem".into(), &"realize".into());
                let fill: &str = &self.fill;
                canvas.set_fill_style(&fill.into());
                canvas.fill_with_path_2d(&Path2d::new_with_path_string(&path.d).unwrap());
            }
            GlyphItem::Image(_path) => {
                // todo!()
            }
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

    #[cfg(feature = "flat-vector")]
    pub module: &'m Module,

    /// A fingerprint builder for generating unique id.
    pub(crate) fingerprint_builder: &'t mut FingerprintBuilder,

    /// Stores the glyphs used in the document.
    pub(crate) glyph_defs: &'t mut GlyphPackBuilder,

    /// See [`ExportFeature`].
    pub should_render_text_element: bool,
    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,

    pub _feat_phantom: std::marker::PhantomData<Feat>,
    #[cfg(not(feature = "flat-vector"))]
    pub _m_phantom: std::marker::PhantomData<&'m ()>,
}

/// A builder for [`SvgTextNode`].
/// It holds a reference to [`SvgRenderTask`] and state of the building process.
pub struct CanvasStack {
    pub ts: sk::Transform,
    pub clipper: Option<ir::PathItem>,
    pub fill: Option<ImmutStr>,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

impl From<CanvasStack> for CanvasNode {
    fn from(s: CanvasStack) -> Self {
        Arc::new(Box::new(CanvasGroupElem {
            ts: s.ts,
            inner: s.inner,
        }))
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
        glyph: &GlyphRef,
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
    fn transform_matrix(mut self, ctx: &mut C, m: &ir::Transform) -> Self {
        let sub_ts: sk::Transform = (*m).into();
        self.ts = self.ts.post_concat(sub_ts);
        self
    }

    fn transform_translate(mut self, ctx: &mut C, matrix: Axes<Abs>) -> Self {
        self.ts = self.ts.post_translate(matrix.x.0, matrix.y.0);
        self
    }

    fn transform_scale(mut self, ctx: &mut C, x: Ratio, y: Ratio) -> Self {
        self.ts = self.ts.post_scale(x.0, y.0);
        self
    }

    fn transform_rotate(self, ctx: &mut C, _matrix: Scalar) -> Self {
        todo!()
    }

    fn transform_skew(mut self, ctx: &mut C, matrix: (Ratio, Ratio)) -> Self {
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

    fn transform_clip(mut self, ctx: &mut C, matrix: &ir::PathItem) -> Self {
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

    fn render_path(&mut self, ctx: &mut C, path: &ir::PathItem) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(Box::new(CanvasPathElem {
                path_data: path.clone(),
            })),
        ))
    }

    fn render_image(&mut self, ctx: &mut C, image_item: &ir::ImageItem) {
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
    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> GlyphRef {
        self.glyph_defs.build_glyph(glyph).0
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

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// fork a render task with module.
    pub fn fork_canvas_render_task<'m, 't>(
        &'t mut self,
        module: &'m flat_ir::Module,
    ) -> CanvasRenderTask<'m, 't, Feat> {
        CanvasRenderTask::<Feat> {
            glyph_provider: self.glyph_provider.clone(),

            module,

            fingerprint_builder: &mut self.fingerprint_builder,

            glyph_defs: &mut self.glyph_defs,

            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct IncrementalCanvasExporter {
    pub pages: Vec<(Arc<Box<dyn CanvasElem>>, Size)>,
}

impl IncrementalCanvasExporter {
    pub fn interpret_changes(&mut self, module: &Module, pages: &[Page]) {
        // render the document
        let mut t = SvgTask::<DefaultExportFeature>::default();

        let mut ct = t.fork_canvas_render_task(&module);

        let pages = pages
            .iter()
            .map(|Page { content, size }| (ct.render_flat_item(content), size.clone()))
            .collect();
        self.pages = pages;
    }

    pub async fn flush_page(&mut self, idx: usize, canvas: &web_sys::CanvasRenderingContext2d) {
        let pg = &self.pages[idx];
        pg.0.realize(sk::Transform::from_scale(3.5, 3.5), canvas)
            .await;
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
            let t = &self.doc.layouts[0];
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
}