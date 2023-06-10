use typst_ts_core::error::prelude::*;

use crate::{
    ir::{PathItem, PathStyle},
    utils::AbsExt,
    RenderFeature, SvgRenderTask,
};

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Render a geometrical shape into the canvas.
    pub(crate) fn render_path(&mut self, path: &PathItem) -> ZResult<String> {
        let mut p = vec!["<path ".to_owned()];
        p.push(format!(r#"d="{}" "#, path.d));
        for style in &path.styles {
            match style {
                PathStyle::Fill(color) => {
                    p.push(format!(r#"fill="{}" "#, color));
                }
                PathStyle::Stroke(color) => {
                    p.push(format!(r#"stroke="{}" "#, color));
                }
                PathStyle::StrokeWidth(width) => {
                    p.push(format!(r#"stroke-width="{}" "#, width.to_f32()));
                }
                PathStyle::StrokeLineCap(cap) => {
                    p.push(format!(r#"stroke-linecap="{}" "#, cap));
                }
                PathStyle::StrokeLineJoin(join) => {
                    p.push(format!(r#"stroke-linejoin="{}" "#, join));
                }
                PathStyle::StrokeMitterLimit(limit) => {
                    p.push(format!(r#"stroke-miterlimit="{}" "#, limit.0));
                }
                PathStyle::StrokeDashArray(array) => {
                    p.push(r#"stroke-dasharray="#.to_owned());
                    for (i, v) in array.iter().enumerate() {
                        if i > 0 {
                            p.push(" ".to_owned());
                        }
                        p.push(format!("{}", v.to_f32()));
                    }
                    p.push(r#"" "#.to_owned());
                }
                PathStyle::StrokeDashOffset(offset) => {
                    p.push(format!(r#"stroke-dashoffset="{}" "#, offset.to_f32()));
                }
            }
        }
        p.push("/>".to_owned());
        let p = p.join("");
        Ok(p)
    }
}
