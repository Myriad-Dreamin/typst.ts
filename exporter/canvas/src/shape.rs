use ttf_parser::OutlineBuilder;
use typst::geom::{Geometry, LineCap, LineJoin, Paint, PathItem, Shape, Stroke};
use typst_ts_core::error::prelude::*;
use web_sys::Path2d;

use crate::{
    sk,
    svg::SvgPath2DBuilder,
    utils::{AbsExt, CanvasStateGuard},
    CanvasRenderTask,
};

impl<'a> CanvasRenderTask<'a> {
    /// Render a geometrical shape into the canvas.
    pub(crate) fn render_shape(&mut self, ts: sk::Transform, shape: &Shape) -> ZResult<()> {
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
                        PathItem::MoveTo(p) => {
                            builder.move_to(p.x.to_f32(), p.y.to_f32());
                        }
                        PathItem::LineTo(p) => {
                            builder.line_to(p.x.to_f32(), p.y.to_f32());
                        }
                        PathItem::CubicTo(p1, p2, p3) => {
                            builder.curve_to(
                                p1.x.to_f32(),
                                p1.y.to_f32(),
                                p2.x.to_f32(),
                                p2.y.to_f32(),
                                p3.x.to_f32(),
                                p3.y.to_f32(),
                            );
                        }
                        PathItem::ClosePath => {
                            builder.close();
                        }
                    };
                }
            }
        };

        // todo: anti_alias
        if let Some(fill) = &shape.fill {
            let state_guard = CanvasStateGuard::new(self.canvas);

            let Paint::Solid(color) = fill;
            let c = color.to_rgba();
            let fill_style = format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a);

            #[cfg(feature = "debug_shape_fill")]
            console_log!(
                "fill pure background {} -> {} [{:?}]",
                builder.0,
                fill_style,
                ts
            );

            self.canvas.set_fill_style(&fill_style.into());
            self.reset_transform();
            self.sync_transform(ts);

            self.canvas.fill_with_path_2d(
                &Path2d::new_with_path_string(&builder.0)
                    .map_err(map_err("CanvasRenderTask.BuildPath2d"))?,
            );

            drop(state_guard)
        } else if let Some(Stroke {
            paint,
            thickness,
            line_cap,
            line_join,
            dash_pattern,
            miter_limit,
        }) = &shape.stroke
        {
            // todo: dash pattern
            let _ = dash_pattern;
            // dash_pattern.as_ref().and_then(|pattern| {
            // tiny-skia only allows dash patterns with an even number of elements,
            // while pdf allows any number.
            // let len = if pattern.array.len() % 2 == 1 {
            //     pattern.array.len() * 2
            // } else {
            //     pattern.array.len()
            // };
            // let dash_array = pattern
            //     .array
            //     .iter()
            //     .map(|l| l.to_f32())
            //     .cycle()
            //     .take(len)
            //     .collect();

            // sk::StrokeDash::new(dash_array, pattern.phase.to_f32())
            //     panic!("dash_pattern");

            // });

            let state_guard = CanvasStateGuard::new(self.canvas);
            self.canvas.set_line_width(thickness.to_pt());
            self.canvas.set_line_cap(match line_cap {
                LineCap::Butt => "butt",
                LineCap::Round => "round",
                LineCap::Square => "square",
            });
            self.canvas.set_line_join(match line_join {
                LineJoin::Miter => "miter",
                LineJoin::Bevel => "bevel",
                LineJoin::Round => "round",
            });
            self.canvas.set_miter_limit(miter_limit.0);

            let Paint::Solid(color) = paint;
            let c = color.to_rgba();
            let fill_style = format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a);
            self.canvas.set_stroke_style(&fill_style.into());

            // todo: ts
            self.set_transform(ts);

            self.canvas.stroke_with_path(
                &Path2d::new_with_path_string(&builder.0)
                    .map_err(map_err("CanvasRenderTask.BuildPath2d"))?,
            );

            drop(state_guard)
        }

        Ok(())
    }
}
