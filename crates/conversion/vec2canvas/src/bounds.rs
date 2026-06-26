use core::fmt;
use std::cell::OnceCell;

use elsa::FrozenMap;
use reflexo::vector::ir::{self, FlatGlyphItem, Rect, Scalar};
use tiny_skia as sk;

use crate::ops::*;

pub trait BBoxAt {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect>;
}

#[derive(Debug, Clone, Copy)]
pub struct CanvasBound {
    pub kind: &'static str,
    pub rect: Rect,
}

pub fn hit_canvas_bound_at(
    node: &CanvasNode,
    ts: sk::Transform,
    point: ir::Point,
) -> Option<CanvasBound> {
    hit_elem_bound_at(node, ts, point)
}

fn hit_elem_bound_at(
    node: &CanvasNode,
    ts: sk::Transform,
    point: ir::Point,
) -> Option<CanvasBound> {
    match node.as_ref() {
        CanvasElem::Group(group) => hit_group_bound_at(group, ts, point),
        CanvasElem::Clip(clip) => hit_clip_bound_at(clip, ts, point),
        CanvasElem::Path(path) => {
            if path.fill.is_none() && path.stroke.is_none() {
                return None;
            }

            let rect = path.bbox_at(ts)?;
            rect_contains(rect, point).then_some(CanvasBound { kind: "path", rect })
        }
        CanvasElem::Image(image) => {
            let rect = image.bbox_at(ts)?;
            rect_contains(rect, point).then_some(CanvasBound {
                kind: "image",
                rect,
            })
        }
        CanvasElem::Glyph(..) => None,
    }
}

fn hit_group_bound_at(
    group: &CanvasGroupElem,
    ts: sk::Transform,
    point: ir::Point,
) -> Option<CanvasBound> {
    if matches!(group.kind, GroupKind::Text) {
        return None;
    }

    let ts = ts.pre_concat(*group.ts.as_ref());
    for (pos, elem) in group.inner.iter().rev() {
        let ts = ts.pre_translate(pos.x.0, pos.y.0);
        if let Some(bound) = hit_elem_bound_at(elem, ts, point) {
            return Some(bound);
        }
    }

    None
}

fn hit_clip_bound_at(
    clip: &CanvasClipElem,
    ts: sk::Transform,
    point: ir::Point,
) -> Option<CanvasBound> {
    if let Some(clip_rect) = clip.clip_bbox_at(ts) {
        if !rect_contains(clip_rect, point) {
            return None;
        }
    }

    hit_elem_bound_at(&clip.inner, ts, point)
}

fn rect_contains(rect: Rect, point: ir::Point) -> bool {
    point.x >= rect.left()
        && point.x <= rect.right()
        && point.y >= rect.top()
        && point.y <= rect.bottom()
}

pub enum CanvasBBox {
    Static(Box<Rect>),
    Dynamic(Box<OnceCell<elsa::FrozenMap<ir::Transform, Box<Option<Rect>>>>>),
}

impl fmt::Debug for CanvasBBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanvasBBox::Static(r) => write!(f, "Static({r:?})"),
            CanvasBBox::Dynamic(..) => write!(f, "Dynamic(..)"),
        }
    }
}

impl CanvasBBox {
    pub fn bbox_at(
        &self,
        ts: sk::Transform,
        compute: impl FnOnce() -> Option<Rect>,
    ) -> Option<Rect> {
        match self {
            CanvasBBox::Static(r) => {
                if ts.is_identity() {
                    return Some(**r);
                }

                let r = sk::Rect::from_xywh(r.lo.x.0, r.lo.y.0, r.width().0, r.height().0)
                    .and_then(|e| e.transform(ts));
                r.map(From::from)
            }
            CanvasBBox::Dynamic(map) => {
                let map = map.get_or_init(FrozenMap::new);
                let ts_key: ir::Transform = ts.into();
                if let Some(r) = map.get(&ts_key) {
                    return *r;
                }

                let r = compute();
                map.insert(ts_key, Box::new(r));

                r
            }
        }
    }
}

impl BBoxAt for CanvasElem {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        match self {
            CanvasElem::Group(g) => g.bbox_at(ts),
            CanvasElem::Clip(g) => g.bbox_at(ts),
            CanvasElem::Path(g) => g.bbox_at(ts),
            CanvasElem::Image(g) => g.bbox_at(ts),
            CanvasElem::Glyph(g) => g.bbox_at(ts),
        }
    }
}

impl BBoxAt for CanvasGroupElem {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        let ts = ts.pre_concat(*self.ts.as_ref());

        self.rect.bbox_at(ts, || {
            self.inner
                .iter()
                .fold(None, |acc: Option<Rect>, (pos, elem)| {
                    // we try to move the bbox instead of concat the translate to ts
                    let Some(r) = elem.bbox_at(ts) else {
                        return acc;
                    };

                    // scale the movement
                    let pos = ir::Point::new(
                        Scalar(pos.x.0 * ts.sx + pos.y.0 * ts.kx),
                        Scalar(pos.y.0 * ts.sy + pos.x.0 * ts.ky),
                    );

                    let r = r.translate(pos);
                    match acc {
                        Some(acc) => Some(acc.union(&r)),
                        None => Some(r),
                    }
                })
        })
    }
}

impl BBoxAt for CanvasClipElem {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        // todo: clip path
        self.inner.bbox_at(ts)
    }
}

impl BBoxAt for CanvasPathElem {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        self.rect.bbox_at(ts, || {
            reflexo_vec2bbox::Vec2BBoxPass::path_bbox(&self.path_data, ts)
        })
    }
}

impl BBoxAt for CanvasImageElem {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        let bbox = sk::Rect::from_xywh(0., 0., self.image_data.size.x.0, self.image_data.size.y.0)
            .and_then(|e| e.transform(ts));

        bbox.map(From::from)
    }
}

impl BBoxAt for CanvasGlyphElem {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect> {
        let rect = self
            .bbox
            .get_or_init(|| glyph_local_bbox(self.glyph_data.as_ref()));
        transform_rect((*rect)?, ts)
    }
}

fn glyph_local_bbox(glyph: &FlatGlyphItem) -> Option<Rect> {
    match glyph {
        FlatGlyphItem::None => None,
        FlatGlyphItem::Image(image) => {
            let rect = sk::Rect::from_xywh(0., 0., image.image.size.x.0, image.image.size.y.0)?;
            rect.transform(image.ts.into()).map(From::from)
        }
        FlatGlyphItem::Outline(outline) => {
            let mut path = convert_path(&outline.d)?;
            if let Some(transform) = &outline.ts {
                let transform: tiny_skia_path::Transform = (**transform).into();
                path = path.transform(transform)?;
            }

            Some(path.bounds().into())
        }
    }
}

fn transform_rect(rect: Rect, ts: sk::Transform) -> Option<Rect> {
    sk::Rect::from_xywh(rect.lo.x.0, rect.lo.y.0, rect.width().0, rect.height().0)
        .and_then(|rect| rect.transform(ts))
        .map(From::from)
}

fn convert_path(path_data: &str) -> Option<tiny_skia_path::Path> {
    let mut builder = tiny_skia_path::PathBuilder::new();

    for segment in svgtypes::SimplifyingPathParser::from(path_data) {
        let segment = segment.ok()?;

        match segment {
            svgtypes::SimplePathSegment::MoveTo { x, y } => builder.move_to(x as f32, y as f32),
            svgtypes::SimplePathSegment::LineTo { x, y } => builder.line_to(x as f32, y as f32),
            svgtypes::SimplePathSegment::Quadratic { x1, y1, x, y } => {
                builder.quad_to(x1 as f32, y1 as f32, x as f32, y as f32)
            }
            svgtypes::SimplePathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => builder.cubic_to(
                x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
            ),
            svgtypes::SimplePathSegment::ClosePath => builder.close(),
        }
    }

    builder.finish()
}
