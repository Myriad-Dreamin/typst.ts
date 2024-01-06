use std::sync::Arc;
use std::{collections::HashMap, ops::Deref};

use comemo::Prehashed;
use typst::text::Font;

use super::{
    ir::{
        self, Abs, Axes, BuildGlyph, FontIndice, FontRef, GlyphIndice, GlyphPackBuilder, GlyphRef,
        Module, Ratio, Rect, Scalar, Transform,
    },
    vm::{
        FlatGroupContext, FlatIncrGroupContext, FlatIncrRenderVm, FlatRenderVm, GroupContext,
        RenderState, RenderVm, TransformContext,
    },
};
use crate::font::GlyphProvider;
use crate::hash::Fingerprint;

/// Use types from `tiny-skia` crate.
use tiny_skia as sk;

pub trait BBoxIndice {
    fn get_bbox(&self, value: &Fingerprint) -> Option<BBox>;
}

pub trait ObservableBounds {
    fn realize(&self, ts: Transform) -> Rect;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathRepr {
    repr: tiny_skia_path::Path,
    data: String,
}

impl PathRepr {
    #[inline]
    fn from_path_data(d: &str) -> Option<Self> {
        convert_path(d).map(|repr| PathRepr {
            repr,
            data: d.to_owned(),
        })
    }

    #[inline]
    fn from_item(p: &ir::PathItem) -> Option<Self> {
        // todo: stroke
        Self::from_path_data(&p.d)
    }
}

impl std::hash::Hash for PathRepr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl ObservableBounds for PathRepr {
    fn realize(&self, ts: Transform) -> Rect {
        let path = self.repr.clone().transform(ts.into());
        path.map(|p| p.bounds().into()).unwrap_or_else(Rect::empty)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct BBox(Arc<Prehashed<BBoxRepr>>);

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum BBoxRepr {
    Group(Transform, Arc<[(ir::Point, BBox)]>),
    Clip((Box<PathRepr>, BBox)),
    Transform((Transform, BBox)),
    Rect(Rect),
    Node(Box<PathRepr>),
}

impl BBox {
    pub fn new(repr: BBoxRepr) -> Self {
        Self(Arc::new(Prehashed::new(repr)))
    }
}

impl ObservableBounds for BBox {
    #[comemo::memoize]
    fn realize(&self, ts: Transform) -> Rect {
        match &self.0.deref().deref() {
            BBoxRepr::Group(group_ts, items) => {
                let ts = ts.pre_concat(*group_ts);

                let mut rect = Rect::empty();

                for (pos, bbox) in items.iter() {
                    let ts = ts.pre_translate(pos.x.0, pos.y.0);
                    let bbox_rect = bbox.realize(ts);
                    rect = rect.union(&bbox_rect);
                }

                rect
            }
            BBoxRepr::Clip((clip_path, bbox)) => {
                // todo: irregular clip path
                let clip_path = clip_path.realize(ts);
                let bbox_rect = bbox.realize(ts);
                bbox_rect.intersect(&clip_path)
            }
            BBoxRepr::Transform((group_ts, bbox)) => bbox.realize(ts.pre_concat(*group_ts)),
            BBoxRepr::Rect(rect) => {
                let mut rect = [rect.lo, rect.hi].map(From::from);
                let ts: tiny_skia_path::Transform = ts.into();
                ts.map_points(&mut rect);

                tiny_skia_path::Rect::from_points(rect.as_slice())
                    .map(From::from)
                    .unwrap_or_else(Rect::empty)
            }
            BBoxRepr::Node(path) => path.realize(ts),
        }
    }
}

/// A builder for [`BBox`].
pub struct BBoxBuilder {
    pub ts: sk::Transform,
    pub clipper: Option<ir::PathItem>,
    pub inner: Vec<(ir::Point, BBox)>,
}

impl From<BBoxBuilder> for BBox {
    fn from(s: BBoxBuilder) -> Self {
        let mut grp = BBox::new(BBoxRepr::Group(s.ts.into(), s.inner.into()));
        if let Some(clipper) = s.clipper {
            grp = BBox::new(BBoxRepr::Clip((
                Box::new(PathRepr::from_item(&clipper).unwrap()),
                grp,
            )));
        }
        grp
    }
}

/// Internal methods for [`BBoxBuilder`].
impl BBoxBuilder {
    pub fn render_glyph_ref_inner(&mut self, pos: Scalar, _id: &GlyphRef, glyph: &ir::GlyphItem) {
        let pos = ir::Point::new(pos, Scalar(0.));
        match glyph {
            ir::GlyphItem::Outline(outline) => {
                let path = PathRepr::from_path_data(&outline.d).unwrap();
                self.inner
                    .push((pos, BBox::new(BBoxRepr::Node(Box::new(path)))))
            }
            ir::GlyphItem::Image(image_item) => self.inner.push((
                pos,
                BBox::new(BBoxRepr::Transform((
                    image_item.ts,
                    BBox::new(BBoxRepr::Rect(Rect {
                        lo: ir::Point::default(),
                        hi: image_item.image.size,
                    })),
                ))),
            )),
            _ => unimplemented!(),
        }
    }
}

/// See [`TransformContext`].
impl<C> TransformContext<C> for BBoxBuilder {
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
impl<C: BuildGlyph + RenderVm<Resultant = BBox>> GroupContext<C> for BBoxBuilder {
    fn render_glyph(&mut self, ctx: &mut C, pos: Scalar, glyph: &ir::GlyphItem) {
        let glyph_ref = ctx.build_glyph(glyph);
        self.render_glyph_ref_inner(pos, &glyph_ref, glyph)
    }

    fn render_path(
        &mut self,
        _state: RenderState,
        _ctx: &mut C,
        path: &ir::PathItem,
        _abs_ref: &Fingerprint,
    ) {
        let path = PathRepr::from_item(path).unwrap();
        self.inner.push((
            ir::Point::default(),
            BBox::new(BBoxRepr::Node(Box::new(path))),
        ))
    }

    fn render_image(&mut self, _ctx: &mut C, image_item: &ir::ImageItem) {
        self.inner.push((
            ir::Point::default(),
            BBox::new(BBoxRepr::Rect(Rect {
                lo: ir::Point::default(),
                hi: image_item.size,
            })),
        ))
    }
}

/// See [`FlatGroupContext`].
impl<'m, C: FlatRenderVm<'m, Resultant = BBox> + GlyphIndice<'m>> FlatGroupContext<C>
    for BBoxBuilder
{
    fn render_item_ref_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        pos: ir::Point,
        item: &Fingerprint,
    ) {
        let bbox = ctx.render_flat_item(state, item);
        self.inner.push((pos, bbox));
    }

    fn render_glyph_ref(&mut self, ctx: &mut C, pos: Scalar, glyph: &GlyphRef) {
        if let Some(glyph_data) = ctx.get_glyph(glyph) {
            self.render_glyph_ref_inner(pos, glyph, glyph_data)
        }
    }
}

/// See [`FlatIncrGroupContext`].
impl<'m, C: FlatIncrRenderVm<'m, Resultant = BBox, Group = BBoxBuilder> + BBoxIndice>
    FlatIncrGroupContext<C> for BBoxBuilder
{
    fn render_diff_item_ref_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        pos: ir::Point,
        item: &Fingerprint,
        prev_item: &Fingerprint,
    ) {
        let bbox = (prev_item == item)
            .then(|| ctx.get_bbox(prev_item))
            .flatten();
        let bbox = bbox.unwrap_or_else(|| ctx.render_diff_item(state, item, prev_item));
        self.inner.push((pos, bbox));
    }
}

/// Task to create bbox with vector IR
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of task.
pub struct BBoxTask<'m, 't> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    pub glyph_provider: GlyphProvider,

    #[cfg(feature = "flat-vector")]
    pub module: &'m Module,

    /// Stores the fonts and glyphs used in the document.
    pub(crate) glyph_defs: &'t mut GlyphPackBuilder,

    /// Stores the bboxes used in the document.
    pub(crate) bbox_cache: &'t mut HashMap<Fingerprint, BBox>,

    #[cfg(not(feature = "flat-vector"))]
    pub _m_phantom: std::marker::PhantomData<&'m ()>,
}

impl<'m, 't> FlatRenderVm<'m> for BBoxTask<'m, 't> {
    type Resultant = BBox;
    type Group = BBoxBuilder;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&mut self, _v: &Fingerprint) -> Self::Group {
        Self::Group {
            ts: sk::Transform::identity(),
            clipper: None,
            inner: vec![],
        }
    }

    fn render_flat_item(&mut self, state: RenderState, abs_ref: &Fingerprint) -> Self::Resultant {
        if let Some(bbox) = self.bbox_cache.get(abs_ref) {
            return bbox.clone();
        }

        let bbox = self._render_flat_item(state, abs_ref);
        self.bbox_cache.insert(*abs_ref, bbox.clone());
        bbox
    }
}

impl<'m, 't> FlatIncrRenderVm<'m> for BBoxTask<'m, 't> {
    fn render_diff_item(
        &mut self,
        state: RenderState,
        next_abs_ref: &Fingerprint,
        prev_abs_ref: &Fingerprint,
    ) -> Self::Resultant {
        let bbox = self._render_diff_item(state, next_abs_ref, prev_abs_ref);
        self.bbox_cache.insert(*next_abs_ref, bbox.clone());
        bbox
    }
}

impl BuildGlyph for BBoxTask<'_, '_> {
    fn build_font(&mut self, font: &Font) -> FontRef {
        self.glyph_defs.build_font(font)
    }

    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> GlyphRef {
        self.glyph_defs.build_glyph(glyph)
    }
}

impl<'m> FontIndice<'m> for BBoxTask<'m, '_> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize)
    }
}

impl<'m> GlyphIndice<'m> for BBoxTask<'m, '_> {
    fn get_glyph(&self, g: &GlyphRef) -> Option<&'m ir::GlyphItem> {
        self.module.glyphs.get(g.glyph_idx as usize).map(|v| &v.1)
    }
}

impl<'m> BBoxIndice for BBoxTask<'m, '_> {
    fn get_bbox(&self, value: &Fingerprint) -> Option<BBox> {
        self.bbox_cache.get(value).cloned()
    }
}

fn convert_path(path_data: &str) -> Option<tiny_skia_path::Path> {
    let mut builder = tiny_skia_path::PathBuilder::new();
    for segment in svgtypes::SimplifyingPathParser::from(path_data) {
        let segment = match segment {
            Ok(v) => v,
            Err(_) => break,
        };

        match segment {
            svgtypes::SimplePathSegment::MoveTo { x, y } => {
                builder.move_to(x as f32, y as f32);
            }
            svgtypes::SimplePathSegment::LineTo { x, y } => {
                builder.line_to(x as f32, y as f32);
            }
            svgtypes::SimplePathSegment::Quadratic { x1, y1, x, y } => {
                builder.quad_to(x1 as f32, y1 as f32, x as f32, y as f32);
            }
            svgtypes::SimplePathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                builder.cubic_to(
                    x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
                );
            }
            svgtypes::SimplePathSegment::ClosePath => {
                builder.close();
            }
        }
    }

    builder.finish()
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {

    pub use super::*;

    #[derive(Default)]
    struct BBoxRenderer {
        glyph_provider: GlyphProvider,
        module: Module,
        glyph_defs: GlyphPackBuilder,
        bbox_cache: HashMap<Fingerprint, BBox>,
    }

    impl BBoxRenderer {
        fn get(&mut self) -> BBoxTask<'_, '_> {
            BBoxTask {
                glyph_provider: self.glyph_provider.clone(),
                module: &self.module,
                glyph_defs: &mut self.glyph_defs,
                bbox_cache: &mut self.bbox_cache,
            }
        }
    }

    // todo
    // fn get_rect_item(x: f32, y: f32, width: f32, height: f32) -> ir::SvgItem
    // {     let mut d = SvgPath2DBuilder::default();
    //     d.rect(x, y, width, height);
    //     let d = d.0.into();
    //     let path = PathItem {
    //         d,
    //         size: None,
    //         styles: Default::default(),
    //     };

    //     ir::SvgItem::Path((path, 0))
    // }

    // todo
    // #[test]
    // fn test_rect_bbox() {
    //     let mut t = BBoxRenderer::default();
    //     let mut task = t.get();

    //     let rect = get_rect_item(1., 2., 10., 20.);
    //     let bbox = task.render_item(
    //         RenderState::new_size(Size::new(Scalar(10.), Scalar(20.))),
    //         &rect,
    //     );

    //     println!("{:?}", bbox.realize(Transform::identity()));
    // }

    // #[test]
    // fn test_transformed_rect_bbox() {
    //     let mut t = BBoxRenderer::default();
    //     let mut task = t.get();

    //     let rect = get_rect_item(1., 2., 10., 20.);
    //     let bbox = task.render_item(
    //         RenderState::new_size(Size::new(Scalar(10.), Scalar(20.))),
    //         &rect,
    //     );

    //     let ts = sk::Transform::from_translate(10., 20.);
    //     println!("{:?}", bbox.realize(ts.into()));

    //     let ts = sk::Transform::from_scale(2., 5.);
    //     println!("{:?}", bbox.realize(ts.into()));

    //     let ts = sk::Transform::from_skew(1.1, 1.7);
    //     println!("{:?}", bbox.realize(ts.into()));
    // }
}
