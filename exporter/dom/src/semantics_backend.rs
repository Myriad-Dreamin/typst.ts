use std::borrow::Cow;

use typst_ts_core::hash::Fingerprint;
use typst_ts_svg_exporter::{
    ir::{self, VecItem},
    Module,
};

use crate::escape::{self, TextContentDataEscapes};

#[derive(Default)]
pub struct SemanticsBackend {}

impl SemanticsBackend {
    pub fn render_semantics<'a>(
        &mut self,
        ctx: &'a Module,
        ts: tiny_skia::Transform,
        fg: Fingerprint,
        output: &mut Vec<Cow<'a, str>>,
    ) {
        let item = ctx.get_item(&fg).unwrap();
        use VecItem::*;
        match item {
            Group(t, _) => {
                output.push(Cow::Borrowed(r#"<span class="typst-content-group""#));
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{}" style="font-size: 0px">"#,
                        ts.tx, ts.ty,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{},{},{},{},{}" style="font-size: 0px">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty,
                    )));
                }
                for (p, child) in t.0.iter() {
                    self.render_semantics(
                        ctx,
                        tiny_skia::Transform::from_translate(p.x.0, p.y.0),
                        *child,
                        output,
                    );
                }
                output.push(Cow::Borrowed("</span>"));
            }
            Item(t) => {
                output.push(Cow::Borrowed(r#"<span class="typst-content-group""#));
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{}" style="font-size: 0px">"#,
                        ts.tx, ts.ty,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{},{},{},{},{}" style="font-size: 0px">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty,
                    )));
                }
                let trans = t.0.clone();
                let trans: ir::Transform = trans.into();
                self.render_semantics(ctx, trans.into(), t.1, output);
                output.push(Cow::Borrowed("</span>"));
            }
            Text(t) => {
                // output.push(Cow::Borrowed(r#"<span>"#));
                // with data-translate
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                let size = t.shape.size.0;
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#"<span data-matrix="{},{}" style="font-size: {}">"#,
                        ts.tx,
                        ts.ty - size,
                        size,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#"<span data-matrix="{},{},{},{},{},{}" style="font-size: {}">"#,
                        ts.sx,
                        ts.ky,
                        ts.kx,
                        ts.sy,
                        ts.tx,
                        ts.ty - size * ts.sy,
                        size,
                    )));
                }
                output.push(escape::escape_str::<TextContentDataEscapes>(
                    t.content.content.as_ref(),
                ));
                output.push(Cow::Borrowed("</span>"));
            }
            ContentHint(c) => {
                output.push(Cow::Borrowed(r#"<span class="typst-content-hint""#));
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{}" style="font-size: 0px">"#,
                        ts.tx, ts.ty,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{},{},{},{},{}" style="font-size: 0px">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty,
                    )));
                }
                let c = c.to_string();
                let c = escape::escape_str::<TextContentDataEscapes>(&c).into_owned();
                output.push(Cow::Owned(c));
                output.push(Cow::Borrowed("</span>"));
            }
            Link(t) => {
                // output.push(Cow::Owned(format!(
                //     r#"<a href="{:?}"></a>"#,
                //     escape::escape_str::<PcDataEscapes>(&t.href),
                // )));
            }
            Image(..) | Path(..) => {}
            None | Gradient(..) | Color32(..) | Pattern(..) => {}
        }
    }
}
