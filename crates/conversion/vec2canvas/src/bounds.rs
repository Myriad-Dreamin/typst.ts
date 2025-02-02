use core::fmt;
use std::cell::OnceCell;

use elsa::FrozenMap;
use reflexo::vector::ir::{self, Rect, Scalar};
use tiny_skia as sk;

use crate::ops::*;

pub trait BBoxAt {
    fn bbox_at(&self, ts: sk::Transform) -> Option<Rect>;
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
    fn bbox_at(&self, _ts: sk::Transform) -> Option<Rect> {
        None
    }
}
