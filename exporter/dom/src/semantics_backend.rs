use std::borrow::Cow;

use typst_ts_canvas_exporter::CanvasStateGuard;
use typst_ts_core::hash::Fingerprint;
use typst_ts_svg_exporter::{
    ir::{self, Scalar, VecItem},
    Module,
};
use web_sys::{wasm_bindgen::JsCast, HtmlCanvasElement};

use crate::escape::{self, AttributeEscapes, TextContentDataEscapes};

#[derive(Clone, Copy)]
pub struct BrowserFontMetric {
    width: f32,
    // height: f32,
}

impl BrowserFontMetric {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        let _g = CanvasStateGuard::new(&ctx);
        ctx.set_font("128px monospace");
        let metrics = ctx.measure_text("A").unwrap();
        let a_width = metrics.width();
        // let a_height =
        //     (metrics.font_bounding_box_descent() +
        // metrics.font_bounding_box_ascent()).abs();

        Self {
            width: (a_width / 128.) as f32,
            // height: (a_height / 128.) as f32,
        }
    }
}

pub struct SemanticsBackend {
    heavy: bool,
    font_metric: BrowserFontMetric,
    width: f32,
    previous_x_text: std::collections::BTreeMap<Scalar, std::collections::BTreeSet<Scalar>>,
    previous_y_text: std::collections::BTreeMap<Scalar, std::collections::BTreeSet<Scalar>>,
    previous_y2_text: std::collections::BTreeSet<Scalar>,
}

impl SemanticsBackend {
    pub fn new(heavy: bool, font_metric: BrowserFontMetric, width: f32) -> Self {
        SemanticsBackend {
            heavy,
            font_metric,
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
                let can_heavy = is_regular_skew && is_regular_scale && self.heavy;
                let size = (t.shape.size) * Scalar(ts.sy);

                let font = ctx.get_font(&t.shape.font).unwrap();
                let cap_height = font.cap_height * size;
                let width = t.width();
                let scale_x = width.0
                    / (self.font_metric.width * size.0 * t.content.content.chars().count() as f32);
                // let scale_y = (size.0 + descender.0) / (size.0 * self.font_metric.height);
                // web_sys::console::log_1(
                //     &format!(
                //         "scale: {:?} {:?} {:?} {:?}",
                //         cap_height, size, descender, ascender
                //     )
                //     .into(),
                // );

                let tx = Scalar(ts.tx);
                let ty = Scalar(ts.ty) - cap_height;
                let ty2 = ty + size;
                let tx2 = tx + width;

                if can_heavy {
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
                        let x_set = t.1.range(..tx2).max();
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
                        r#"<span class="typst-content-text" style="font-size: calc(var(--data-text-height) * {}); line-height: calc(var(--data-text-height) * {}); left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); transform: scaleX({})">"#,
                        size.0,
                        size.0,
                        tx.0,
                        ty.0,
                        scale_x,
                        // scale_y,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-text" data-matrix="{},{},{},{}" style="font-size: {}px; line-height: calc(var(--data-text-height) * {}); left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); transform: scaleX({})">"#,
                        ts.sx,
                        ts.ky,
                        ts.kx,
                        ts.sy,
                        size.0,
                        size.0,
                        tx.0,
                        ty.0,
                        scale_x,
                        // scale_y,
                    )));
                }

                output.push(escape::escape_str::<TextContentDataEscapes>(
                    t.content.content.as_ref(),
                ));
                output.push(Cow::Borrowed("</span>"));

                if can_heavy {
                    if ty > ty2 {
                        web_sys::console::log_1(
                            &format!(
                                "ty..ty2: {:?} {:?} {:?} {:?} {:?}",
                                font.family,
                                ty..ty2,
                                size,
                                font.descender,
                                ts,
                            )
                            .into(),
                        );
                    }

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
                } else {
                    let mut u = [
                        tiny_skia::Point::from_xy(tx.0, ty.0),
                        tiny_skia::Point::from_xy(tx2.0, ty2.0),
                    ];
                    ts.map_points(&mut u);
                    let tx = Scalar(u[0].x);
                    let ty = Scalar(u[0].y);
                    let tx2 = Scalar(u[1].x);
                    let ty2 = Scalar(u[1].y);

                    let ty_bucket = self.previous_x_text.entry(tx).or_default();
                    ty_bucket.insert(ty);
                    ty_bucket.insert(ty2);
                    let ty_bucket = self.previous_x_text.entry(tx2).or_default();
                    ty_bucket.insert(ty);
                    ty_bucket.insert(ty2);

                    let tx_bucket = self.previous_y_text.entry(ty).or_default();
                    tx_bucket.insert(tx);
                    tx_bucket.insert(tx2);
                    let tx_bucket = self.previous_y_text.entry(ty2).or_default();
                    tx_bucket.insert(tx);
                    tx_bucket.insert(tx2);

                    self.previous_y2_text.insert(ty);
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
                let href_handler = if t.href.starts_with("@typst:") {
                    let href = t.href.trim_start_matches("@typst:");
                    format!(r##" onclick="{href}; return false""##)
                } else {
                    String::new()
                };
                output.push(Cow::Owned(format!(
                    r#"<a class="typst-content-link" href="{}""#,
                    if href_handler.is_empty() {
                        escape::escape_str::<AttributeEscapes>(&t.href)
                    } else {
                        Cow::Borrowed("#")
                    },
                )));
                if !href_handler.is_empty() {
                    output.push(Cow::Owned(href_handler));
                }
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
