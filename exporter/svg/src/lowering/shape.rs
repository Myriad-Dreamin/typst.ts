use std::sync::Arc;

use ttf_parser::OutlineBuilder;
use typst::geom::{Geometry, LineCap, LineJoin, Paint, PathItem as TypstPathItem, Shape, Stroke};
use typst_ts_core::error::prelude::*;

use crate::{
    ir::{PathItem, PathStyle, SvgItem},
    svg::SvgPath2DBuilder,
    utils::{AbsExt, ToCssExt},
    RenderFeature, SvgRenderTask,
};

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Lower a geometrical shape into item.
    pub(super) fn lower_shape(&mut self, shape: &Shape) -> ZResult<SvgItem> {
        let mut builder = SvgPath2DBuilder(String::new());

        // to ensure that our shape focus on the original point
        builder.move_to(0., 0.);
        match shape.geometry {
            Geometry::Line(target) => {
                builder.line_to(target.x.to_f32(), target.y.to_f32());
            }
            Geometry::Rect(size) => {
                let w = size.x.to_f32();
                let h = size.y.to_f32();
                builder.line_to(0., h);
                builder.line_to(w, h);
                builder.line_to(w, 0.);
                builder.close();
            }
            Geometry::Path(ref path) => {
                for elem in &path.0 {
                    match elem {
                        TypstPathItem::MoveTo(p) => {
                            builder.move_to(p.x.to_f32(), p.y.to_f32());
                        }
                        TypstPathItem::LineTo(p) => {
                            builder.line_to(p.x.to_f32(), p.y.to_f32());
                        }
                        TypstPathItem::CubicTo(p1, p2, p3) => {
                            builder.curve_to(
                                p1.x.to_f32(),
                                p1.y.to_f32(),
                                p2.x.to_f32(),
                                p2.y.to_f32(),
                                p3.x.to_f32(),
                                p3.y.to_f32(),
                            );
                        }
                        TypstPathItem::ClosePath => {
                            builder.close();
                        }
                    };
                }
            }
        };

        let d = builder.0;

        let mut styles = Vec::new();

        if let Some(paint_fill) = &shape.fill {
            let Paint::Solid(color) = paint_fill;
            let c = color.to_css();

            styles.push(PathStyle::Fill(c.into()));
        }

        // todo: default miter_limit, thickness
        if let Some(Stroke {
            paint,
            thickness,
            line_cap,
            line_join,
            dash_pattern,
            miter_limit,
        }) = &shape.stroke
        {
            if let Some(pattern) = dash_pattern.as_ref() {
                styles.push(PathStyle::StrokeDashOffset(pattern.phase));
                styles.push(PathStyle::StrokeDashArray(pattern.array.clone().into()));
            }

            styles.push(PathStyle::StrokeWidth(*thickness));
            styles.push(PathStyle::StrokeMitterLimit(*miter_limit));
            match line_cap {
                LineCap::Butt => {}
                LineCap::Round => styles.push(PathStyle::StrokeLineCap("round".into())),
                LineCap::Square => styles.push(PathStyle::StrokeLineCap("square".into())),
            };
            match line_join {
                LineJoin::Miter => {}
                LineJoin::Bevel => styles.push(PathStyle::StrokeLineJoin("bevel".into())),
                LineJoin::Round => styles.push(PathStyle::StrokeLineJoin("round".into())),
            }

            // todo: default color
            let Paint::Solid(color) = paint;
            styles.push(PathStyle::Stroke(color.to_css().into()));
        }

        Ok(SvgItem::Path(Arc::new(PathItem { d, styles })))
    }
}
