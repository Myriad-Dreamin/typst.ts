use std::borrow::Cow;

use typst_ts_core::hash::Fingerprint;
use typst_ts_svg_exporter::{
    ir::{self, Scalar, VecItem},
    Module,
};

use crate::escape::{self, AttributeEscapes, TextContentDataEscapes};

pub struct SemanticsBackend {
    heavy: bool,
    a_width: f32,
    width: f32,
    previous_x_text: std::collections::BTreeMap<Scalar, std::collections::BTreeSet<Scalar>>,
    previous_y_text: std::collections::BTreeMap<Scalar, std::collections::BTreeSet<Scalar>>,
    previous_y2_text: std::collections::BTreeSet<Scalar>,
}

impl SemanticsBackend {
    pub fn new(heavy: bool, a_width: f32, width: f32) -> Self {
        SemanticsBackend {
            heavy,
            a_width,
            width,
            previous_x_text: std::collections::BTreeMap::new(),
            previous_y_text: std::collections::BTreeMap::new(),
            previous_y2_text: std::collections::BTreeSet::new(),
        }
    }

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
                output.push(Cow::Borrowed(r#"<span class="typst-content-group">"#));
                for (pos, child) in t.0.iter() {
                    let ts = ts.pre_translate(pos.x.0, pos.y.0);
                    self.render_semantics(ctx, ts, *child, output);
                }
                output.push(Cow::Borrowed("</span>"));
            }
            Item(t) => {
                output.push(Cow::Borrowed(r#"<span class="typst-content-group">"#));
                let trans = t.0.clone();
                let trans: ir::Transform = trans.into();
                let ts = ts.pre_concat(trans.into());
                self.render_semantics(ctx, ts, t.1, output);
                output.push(Cow::Borrowed("</span>"));
            }
            Text(t) => {
                // output.push(Cow::Borrowed(r#"<span>"#));
                // with data-translate
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                let size = t.shape.size.0;

                let width = t.width();
                let scale_x =
                    width.0 / (self.a_width * size * t.content.content.chars().count() as f32);

                if self.heavy {
                    let tx = Scalar(ts.tx);
                    let ty2 = Scalar(ts.ty);
                    let ty = ty2 - Scalar(size);

                    let top_bound_y = *self
                        .previous_y2_text
                        .range(..ty)
                        .last()
                        .unwrap_or(&Scalar(0.));

                    let top_bound_set = self.previous_y_text.range(..ty);
                    let mut any_x = Scalar(-1e33);
                    let mut top_bound_y2: Option<Scalar> = Option::None;
                    for t in top_bound_set {
                        let x_set = t.1.range(..tx).last();
                        if let Some(x) = x_set {
                            if x > &any_x {
                                any_x = *x;
                                top_bound_y2 = Some(*t.0);
                            }
                        }
                    }

                    let up_top = top_bound_y
                        .max(top_bound_y2.unwrap_or_default())
                        .max(Scalar(0.));
                    let up_left = if top_bound_y2.is_some() {
                        any_x
                    } else {
                        Scalar(0.)
                    };

                    let up_right = self.width;
                    let up_bottom = ty;

                    // create up rect
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        up_left.0,
                        up_top.0,
                        up_right - up_left.0,
                        up_bottom.0 - up_top.0,
                    )));

                    let top_bound_set = self.previous_y_text.range(..ty2);
                    let mut left_left = Scalar(-1e33);
                    for t in top_bound_set {
                        let x_set = t.1.range(..tx).max();
                        if let Some(x) = x_set {
                            if x > &left_left {
                                left_left = *x;
                            }
                        }
                    }

                    let left_left = left_left.max(Scalar(0.));
                    let left_right = tx;
                    let left_top = ty;
                    let left_bottom = ty2;

                    // create left rect
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        left_left.0,
                        left_top.0,
                        left_right.0 - left_left.0,
                        left_bottom.0 - left_top.0,
                    )));
                }

                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-text" style="font-size: {}px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); transform: scaleX({})">"#,
                        size,
                        ts.tx,
                        (ts.ty -size),
                        scale_x,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-text" data-matrix="{},{},{},{}" style="font-size: {}px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); transform: scaleX({})">"#,
                        ts.sx,
                        ts.ky,
                        ts.kx,
                        ts.sy,
                        size,
                        ts.tx,
                        (ts.ty - size * ts.sy),
                        scale_x,
                    )));
                }

                output.push(escape::escape_str::<TextContentDataEscapes>(
                    t.content.content.as_ref(),
                ));
                output.push(Cow::Borrowed("</span>"));

                if self.heavy {
                    let tx = Scalar(ts.tx);
                    let tx2 = tx + width;

                    let ty2 = Scalar(ts.ty);
                    let ty = ty2 - Scalar(size);

                    let top_bound_set = self.previous_y_text.range(ty..ty2);
                    let mut right_right = Scalar(1e33);
                    for t in top_bound_set {
                        let x_set = t.1.range(tx2..).min();
                        if let Some(x) = x_set {
                            if x < &right_right {
                                right_right = *x;
                            }
                        }
                    }

                    let right_right = right_right.0.min(self.width);
                    let right_left = tx2;
                    let right_top = ty;
                    let right_bottom = ty2;

                    // create right rect
                    output.push(Cow::Owned(format!(
                    r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                    right_left.0,
                    right_top.0,
                    right_right - right_left.0,
                    right_bottom.0 - right_top.0,
                )));

                    self.previous_x_text.entry(tx).or_default().insert(ty2);
                    self.previous_x_text.entry(tx2).or_default().insert(ty2);

                    let tx_bucket = self.previous_y_text.entry(ty).or_default();
                    tx_bucket.insert(tx);
                    tx_bucket.insert(tx2);
                    let tx_bucket = self.previous_y_text.entry(ty2).or_default();
                    tx_bucket.insert(tx);
                    tx_bucket.insert(tx2);

                    self.previous_y2_text.insert(ty2);
                }
            }
            ContentHint(c) => {
                if *c == '\n' {
                    // elem.style.top = `calc(var(--data-text-height) * ${rrt})`;
                    // elem.style.left = `calc(var(--data-text-width) * ${rrl})`;
                    output.push(Cow::Borrowed(r#"<br class="typst-content-hint""#));
                    let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                    let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                    if is_regular_scale && is_regular_skew {
                        output.push(Cow::Owned(format!(
                            r#" style="font-size: 0px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {});">"#,
                            ts.tx,ts.ty,
                        )));
                    } else {
                        output.push(Cow::Owned(format!(
                            r#" data-matrix="{},{},{},{}" style="font-size: 0px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {});">"#,
                            ts.sx, ts.ky, ts.kx, ts.sy,   ts.tx,ts.ty,
                        )));
                    }
                    return;
                }
                output.push(Cow::Borrowed(r#"<span class="typst-content-hint""#));
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#" style="font-size: 0px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {});">"#,
                        ts.tx,ts.ty,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{},{},{}" style="font-size: 0px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {});">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy,   ts.tx,ts.ty,
                    )));
                }
                let c = c.to_string();
                let c = escape::escape_str::<TextContentDataEscapes>(&c).into_owned();
                output.push(Cow::Owned(c));
                output.push(Cow::Borrowed("</span>"));
            }
            Link(t) => {
                output.push(Cow::Owned(format!(
                    r#"<a class="typst-content-link" href="{}""#,
                    escape::escape_str::<AttributeEscapes>(&t.href),
                )));
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#" style="font-size: 0px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {});  width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});">"#,
                        ts.tx - 1., ts.ty - 2., t.size.x.0 + 2., t.size.y.0 + 4.,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{},{},{},{}" style="font-size: 0px; left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {});  width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy, ts.tx,ts.ty, t.size.x.0, t.size.y.0,
                    )));
                }
                output.push(Cow::Borrowed("</a>"));
            }
            Image(..) | Path(..) => {}
            None | Gradient(..) | Color32(..) | Pattern(..) => {}
        }
    }
}
