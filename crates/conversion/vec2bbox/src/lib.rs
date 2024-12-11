use std::collections::HashMap;

use tiny_skia as sk;

use reflexo::{hash::Fingerprint, vector::ir::*};

#[derive(Default)]
pub struct Vec2BBoxPass {
    bbox_caches: HashMap<(Fingerprint, Transform), Option<Rect>>,
}

impl Vec2BBoxPass {
    /// Calculate the bounding box of a vector item with a given transform.
    /// The transform is required to calculate the accurate bounding box for
    /// irregular shapes.
    pub fn bbox_of(&mut self, module: &Module, v: Fingerprint, ts: Transform) -> Option<Rect> {
        if let Some(bbox) = self.bbox_caches.get(&(v, ts)) {
            return *bbox;
        }

        let bbox = self.bbox_of_(module, v, ts);
        println!("bbox_of({v:?}, {ts:?}) = {bbox:?}");
        self.bbox_caches.insert((v, ts), bbox);
        bbox
    }

    fn bbox_of_(&mut self, module: &Module, v: Fingerprint, ts: Transform) -> Option<Rect> {
        let item = module.get_item(&v).unwrap();
        match item {
            VecItem::Item(item) => self.bbox_of(module, item.1, item.0.clone().into()),
            VecItem::Group(g) => {
                let mut r = Rect::default();
                for (p, f) in g.0.iter() {
                    let sub_bbox = self.bbox_of(module, *f, ts);
                    if let Some(sub_bbox) = sub_bbox {
                        union(&mut r, *p, sub_bbox);
                    }
                }
                Some(r)
            }
            VecItem::Image(ImageItem { size, .. })
            | VecItem::Link(LinkItem { size, .. })
            | VecItem::SizedRawHtml(SizedRawHtmlItem { size, .. }) => self.rect(*size, ts),
            // todo: I'm writing this in my leg
            VecItem::Text(t) => {
                let width = t.width();
                let height = t.shape.size.0;
                tiny_skia_path::Rect::from_xywh(0.0, 0.0, width.0, height).map(|e| e.into())
            }
            VecItem::Path(p) => self.path(p, ts),
            VecItem::ContentHint(..)
            | VecItem::ColorTransform(..)
            | VecItem::Pattern(..)
            | VecItem::Gradient(..)
            | VecItem::Color32(..)
            | VecItem::Html(..)
            | VecItem::None => None,
        }
    }

    pub fn path(&mut self, p: &PathItem, ts: Transform) -> Option<Rect> {
        Self::path_bbox(p, ts.into())
    }

    fn rect(&self, size: Axes<Scalar>, ts: Transform) -> Option<Rect> {
        let r = tiny_skia_path::Rect::from_xywh(0.0, 0.0, size.x.0, size.y.0);
        r.and_then(|e| e.transform(ts.into())).map(|e| e.into())
    }

    pub fn simple_path_bbox(p: &str, ts: sk::Transform) -> Option<Rect> {
        let d = convert_path(p);
        d.and_then(|e| e.transform(ts))
            .and_then(|e| e.compute_tight_bounds())
            .map(|e| e.into())
    }

    pub fn path_bbox(p: &PathItem, ts: sk::Transform) -> Option<Rect> {
        let d = convert_path(&p.d);
        d.and_then(|e| e.transform(ts))
            .and_then(|e| e.compute_tight_bounds())
            .and_then(|e| {
                let Some(stroke) = p.styles.iter().find_map(|s| match s {
                    PathStyle::StrokeWidth(w) => Some(w.0),
                    _ => None,
                }) else {
                    return Some(e);
                };
                let sk::Transform { sx, sy, kx, ky, .. } = ts;
                let stroke_x = (stroke * (sx + ky)).abs();
                let stroke_y = (stroke * (sy + kx)).abs();
                // extend the bounding box by the stroke width
                let x = e.x() - stroke_x;
                let y = e.y() - stroke_y;
                let w = e.width() + stroke_x * 2.0;
                let h = e.height() + stroke_y * 2.0;
                tiny_skia_path::Rect::from_xywh(x, y, w, h)
            })
            .map(|e| e.into())
    }
}

fn union(r: &mut Rect, p: Axes<Scalar>, sub_bbox: Rect) {
    *r = r.union(&sub_bbox.translate(p))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_bbox() {
        let data = "M 0 0 M 0 4.8 C 0 2.1490333 2.1490333 0 4.8 0 L 975.2 0 C 977.85095 0 980 2.1490333 980 4.8 L 980 122.256 C 980 124.90697 977.85095 127.056 975.2 127.056 L 4.8 127.056 C 2.1490333 127.056 0 124.90697 0 122.256 Z ";

        let p = PathItem {
            d: data.into(),
            size: None,
            styles: vec![],
        };

        let ts = sk::Transform::from_scale(4.5, 4.5);

        assert!(Vec2BBoxPass::path_bbox(&p, ts).is_some());
    }
}
