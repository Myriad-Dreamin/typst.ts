mod canvas_backend;
mod dom;
mod factory;
mod incr;
mod semantics_backend;
mod svg_backend;

use std::ops::Deref;
use std::sync::Arc;

use comemo::Prehashed;
use reflexo::hash::Fingerprint;
use reflexo::vector::ir::{self, Abs, Axes, FontItem, GlyphRef, Ratio, Rect, Scalar, Transform};
use reflexo::vector::vm::{GroupContext, RenderVm, TransformContext};
use reflexo_typst2vec::ir::GlyphItem;
use tiny_skia as sk;

pub use crate::dom::DomPage;
pub use incr::*;

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
    pub fn render_glyph_inner(&mut self, pos: Scalar, _id: &GlyphRef, glyph: &GlyphItem) {
        let pos = ir::Point::new(pos, Scalar(0.));
        match glyph {
            GlyphItem::Outline(outline) => {
                let path = PathRepr::from_path_data(&outline.d).unwrap();
                self.inner
                    .push((pos, BBox::new(BBoxRepr::Node(Box::new(path)))))
            }
            GlyphItem::Image(image_item) => self.inner.push((
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
impl<'m, C: RenderVm<'m, Resultant = BBox>> GroupContext<C> for BBoxBuilder {
    fn render_glyph(&mut self, _ctx: &mut C, _pos: Axes<Scalar>, _font: &FontItem, _glyph_id: u32) {
        unreachable!();
    }

    fn render_path(&mut self, _ctx: &mut C, path: &ir::PathItem, _abs_ref: &Fingerprint) {
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

    fn render_item_at(&mut self, ctx: &mut C, pos: ir::Point, item: &Fingerprint) {
        let bbox = ctx.render_item(item);
        self.inner.push((pos, bbox));
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
