use std::sync::Arc;

use web_sys::Path2d;

use crate::flat_vector::vm::FlatGroupContext;
use crate::flat_vector::FlatRenderVm;
use crate::font::GlyphProvider;

use crate::ir::{self, Abs, AbsoulteRef, Axes, GlyphItem, ImageItem, ImmutStr, Ratio, Scalar};
use crate::vector::vm::{GroupContext, TransformContext};
use crate::{flat_ir, DefaultExportFeature, SvgDocument, SvgTask};
use crate::{ir::GlyphPackBuilder, sk, ExportFeature, Module};

use async_trait::async_trait;
#[async_trait(?Send)]
pub trait CanvasElem {
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

pub struct CanvasGroupElem {
    pub ts: sk::Transform,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasGroupElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        let ts = ts.post_concat(self.ts);
        for (pos, sub_elem) in &self.inner {
            let ts = ts.post_translate(pos.x.0, pos.y.0);
            sub_elem.realize(ts, canvas).await;
        }
    }
}

pub struct CanvasPathElem {
    pub path_data: ir::PathItem,
}

#[async_trait(?Send)]
impl CanvasElem for CanvasPathElem {
    async fn realize(&self, ts: sk::Transform, canvas: &web_sys::CanvasRenderingContext2d) {
        set_transform(canvas, ts);
        // todo style
        canvas.fill_with_path_2d(&Path2d::new_with_path_string(&self.path_data.d).unwrap());
    }
}

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

        // self.t.canvas.draw_image_with_html_image_element_and_dw_and_dh(, dx, dy, dw, dh)
        todo!()
    }
}

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
                let fill: &str = &self.fill;
                canvas.set_fill_style(&fill.into());
                canvas.fill_with_path_2d(&Path2d::new_with_path_string(&path.d).unwrap());
            }
            GlyphItem::Image(_path) => todo!(),
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

    /// Stores the glyphs used in the document.
    glyph_pack: &'t mut GlyphPackBuilder,

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
pub struct CanvasStack<'s, 'm, 't, Feat: ExportFeature> {
    pub t: &'s mut CanvasRenderTask<'m, 't, Feat>,
    pub ts: sk::Transform,
    pub clipper: Option<ir::PathItem>,
    pub fill: Option<ImmutStr>,
    pub inner: Vec<(ir::Point, CanvasNode)>,
}

impl<'s, 'm, 't, Feat: ExportFeature> From<CanvasStack<'s, 'm, 't, Feat>> for CanvasNode {
    fn from(s: CanvasStack<'s, 'm, 't, Feat>) -> Self {
        Arc::new(Box::new(CanvasGroupElem {
            ts: s.ts,
            inner: s.inner,
        }))
    }
}

/// Internal methods for [`CanvasStack`].
impl<'s, 'm, 't, Feat: ExportFeature> CanvasStack<'s, 'm, 't, Feat> {
    pub fn with_text_shape(&mut self, shape: &ir::TextShape) {
        self.fill = Some(shape.fill.clone())
    }

    pub fn render_glyph_ref_inner(&mut self, pos: Scalar, glyph: &AbsoulteRef) {
        let glyph_data = self.t.module.glyphs[glyph.id.0 as usize].1.clone();
        self.inner.push((
            ir::Point::new(pos, Scalar(0.)),
            Arc::new(Box::new(CanvasGlyphElem {
                fill: self.fill.clone().unwrap(),
                glyph_data,
            })),
        ))
    }
}

/// See [`TransformContext`].
impl<'s, 'm, 't, Feat: ExportFeature> TransformContext for CanvasStack<'s, 'm, 't, Feat> {
    fn transform_matrix(mut self, m: &ir::Transform) -> Self {
        let sub_ts: sk::Transform = (*m).into();
        self.ts = self.ts.post_concat(sub_ts);
        self
    }

    fn transform_translate(mut self, matrix: Axes<Abs>) -> Self {
        self.ts = self.ts.post_translate(matrix.x.0, matrix.y.0);
        self
    }

    fn transform_scale(mut self, x: Ratio, y: Ratio) -> Self {
        self.ts = self.ts.post_scale(x.0, y.0);
        self
    }

    fn transform_rotate(self, _matrix: Scalar) -> Self {
        todo!()
    }

    fn transform_skew(mut self, matrix: (Ratio, Ratio)) -> Self {
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

    fn transform_clip(mut self, matrix: &ir::PathItem) -> Self {
        self.clipper = Some(matrix.clone());
        self
    }
}

/// See [`GroupContext`].
impl<'s, 'm, 't, Feat: ExportFeature> GroupContext for CanvasStack<'s, 'm, 't, Feat> {
    fn render_item_at(&mut self, _pos: ir::Point, _item: &ir::SvgItem) {
        // let ts = self.ts;
        // self.ts = ts.post_translate(pos.x.0, pos.y.0);
        // self.t.render_item(item);
        // self.ts = ts;
        todo!();
    }

    fn render_glyph(&mut self, pos: Scalar, glyph: &ir::GlyphItem) {
        let glyph_ref = self.t.glyph_pack.build_glyph(glyph);
        self.render_glyph_ref_inner(pos, &glyph_ref)
    }

    fn render_path(&mut self, path: &ir::PathItem) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(Box::new(CanvasPathElem {
                path_data: path.clone(),
            })),
        ))
    }

    fn render_image(&mut self, image_item: &ir::ImageItem) {
        self.inner.push((
            ir::Point::default(),
            Arc::new(Box::new(CanvasImageElem {
                image_data: image_item.clone(),
            })),
        ))
    }
}

/// See [`FlatGroupContext`].
impl<'s, 'm, 't, Feat: ExportFeature> FlatGroupContext for CanvasStack<'s, 'm, 't, Feat> {
    fn render_item_ref_at(&mut self, pos: crate::ir::Point, item: &AbsoulteRef) {
        let ts = self.ts;
        self.ts = ts.post_translate(pos.x.0, pos.y.0);
        self.t.render_flat_item(item);
        self.ts = ts;
    }

    fn render_glyph_ref(&mut self, pos: Scalar, glyph: &AbsoulteRef) {
        self.render_glyph_ref_inner(pos, glyph)
    }
}

impl<'s, 'c: 's, 'm: 's, 't: 's, Feat: ExportFeature + 's> FlatRenderVm<'s, 'm>
    for CanvasRenderTask<'m, 't, Feat>
{
    // type Resultant = String;
    type Resultant = CanvasNode;
    type Group = CanvasStack<'s, 'm, 't, Feat>;

    fn get_item(&self, value: &AbsoulteRef) -> Option<&'m flat_ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&'s mut self, _v: &AbsoulteRef) -> Self::Group {
        Self::Group {
            t: self,
            ts: sk::Transform::identity(),
            clipper: None,
            fill: None,
            inner: vec![],
        }
    }

    fn start_flat_text(
        &'s mut self,
        value: &AbsoulteRef,
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

            glyph_pack: &mut self.glyph_pack,

            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct IncrementalCanvasExporter {
    pub pages: Vec<Arc<Box<dyn CanvasElem>>>,
}

impl IncrementalCanvasExporter {
    pub fn interpret_changes(&mut self, diff_doc: SvgDocument) {
        // render the document
        let mut t = SvgTask::<DefaultExportFeature>::default();

        let mut ct = t.fork_canvas_render_task(&diff_doc.module);

        let pages = diff_doc
            .pages
            .iter()
            .map(|(abs_ref, ..)| ct.render_flat_item(abs_ref))
            .collect();
        self.pages = pages;
    }

    pub async fn flush_page(&mut self, idx: usize, canvas: &web_sys::CanvasRenderingContext2d) {
        let pg = &self.pages[idx];
        pg.realize(sk::Transform::from_scale(3.5, 3.5), canvas)
            .await;
    }
}
