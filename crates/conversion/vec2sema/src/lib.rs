mod incr;

use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
};

use reflexo::{
    escape::{self, escape_str, AttributeEscapes, PcDataEscapes},
    hash::Fingerprint,
    vector::ir::{self, Module, Point, Rect, Scalar, VecItem},
};
use reflexo_vec2canvas::BrowserFontMetric;
use unicode_width::UnicodeWidthChar;

pub use incr::*;

pub struct SemaTask {
    heavy: bool,
    font_metric: BrowserFontMetric,
    page_width: f32,
    page_height: f32,
    dfn_count: usize,
    rects: Vec<(Fingerprint, Rect)>,
    discrete_label_map: BTreeMap<Scalar, usize>,
    discrete_value_map: Vec<Scalar>,
}

const EPS: f32 = 1e-3;

impl SemaTask {
    pub fn new(heavy: bool, font_metric: BrowserFontMetric, width: f32, height: f32) -> Self {
        SemaTask {
            heavy,
            font_metric,
            page_width: width,
            page_height: height,
            dfn_count: 0,
            rects: vec![],
            discrete_label_map: BTreeMap::new(),
            discrete_value_map: vec![],
        }
    }

    pub fn render_semantics<'a>(
        &mut self,
        ctx: &'a Module,
        ts: tiny_skia::Transform,
        fg: Fingerprint,
        output: &mut Vec<Cow<'a, str>>,
    ) {
        self.prepare_text_rects(ctx, ts, fg);
        self.prepare_discrete_map();
        let mut fallbacks = self.calc_text_item_fallbacks();
        self.dfn_count = 0;
        self.render_semantics_walk(ctx, ts, fg, &mut fallbacks, output);
    }

    fn prepare_text_rects(&mut self, ctx: &Module, ts: tiny_skia::Transform, fg: Fingerprint) {
        let item = ctx.get_item(&fg).unwrap();
        use VecItem::*;
        match item {
            Group(t) => {
                for (pos, child) in t.0.iter() {
                    let ts = ts.pre_translate(pos.x.0, pos.y.0);
                    self.prepare_text_rects(ctx, ts, *child);
                }
            }
            Item(t) => {
                let trans = t.0.clone();
                let trans: ir::Transform = trans.into();
                let ts = ts.pre_concat(trans.into());
                self.prepare_text_rects(ctx, ts, t.1);
            }
            Labelled(t) => {
                self.prepare_text_rects(ctx, ts, t.1);
            }
            Text(t) => {
                // main logic
                let size = (t.shape.size) * Scalar(ts.sy);

                let font = ctx.get_font(&t.shape.font).unwrap();
                let cap_height = font.cap_height * size;
                let width = t.width();

                let tx = Scalar(ts.tx);
                let ty = Scalar(ts.ty) - cap_height;
                let ty2 = ty + size;
                let tx2 = tx + width;

                self.rects.push((
                    fg,
                    Rect {
                        lo: Point { x: tx, y: ty },
                        hi: Point { x: tx2, y: ty2 },
                    },
                ));
            }
            // todo
            // Html(t) => {
            //     // t.size

            //     let tx = Scalar(ts.tx);
            //     let ty = Scalar(ts.ty);

            //     let tx2 = tx + Scalar(t.size.x.0 * ts.sx + t.size.y.0 * ts.kx);
            //     let ty2 = ty + Scalar(t.size.x.0 * ts.ky + t.size.y.0 * ts.sy);

            //     self.rects.push((
            //         fg,
            //         Rect {
            //             lo: Point { x: tx, y: ty },
            //             hi: Point { x: tx2, y: ty2 },
            //         },
            //     ));
            // }
            _ => {}
        }
    }

    fn prepare_discrete_map(&mut self) {
        let nums = &mut self.discrete_value_map;

        for (_, rect) in self.rects.iter() {
            nums.push(rect.lo.x);
            nums.push(rect.lo.y);
            nums.push(rect.hi.x);
            nums.push(rect.hi.y);
        }

        // page borders
        nums.push(0.0.into());
        nums.push(self.page_width.into());
        // todo: page height

        nums.sort();

        // unique label for f32 pairs
        struct DiscreteState {
            label: usize,
            last: Scalar,
        }
        let mut state = Option::<DiscreteState>::None;

        fn approx_eq(a: f32, b: f32) -> bool {
            // todo: use transform-aware epsilon
            (a - b).abs() < EPS
        }

        for (idx, &mut num) in nums.iter_mut().enumerate() {
            if let Some(state) = state.as_mut() {
                if !approx_eq(state.last.0, num.0) {
                    state.label = idx;
                }
            } else {
                state = Some(DiscreteState {
                    label: idx,
                    last: num,
                });
            }
            let state = state.as_mut().unwrap();
            self.discrete_label_map.insert(num, state.label);
            state.last = num;
        }
    }

    // Vec<(prepend: String, append: String)>
    fn calc_text_item_fallbacks(&mut self) -> VecDeque<(String, String)> {
        let mut res = VecDeque::new();
        res.resize(self.rects.len(), (String::new(), String::new()));

        // Append right and bottom fallbacks
        for (idx, (_, rect)) in self.rects.iter().enumerate() {
            let left = rect.lo.x;
            let top = rect.lo.y;
            let right = rect.hi.x;
            let bottom = rect.hi.y;

            res[idx].1.push_str(&format!(
                r#"<span class="typst-content-fallback typst-content-fallback-rb1" style="left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5}); width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});"></span>"#,
                left.0,
                bottom.0,
                self.page_width - left.0,
                self.page_height - bottom.0,
            ));

            res[idx].1.push_str(&format!(
                r#"<span class="typst-content-fallback typst-content-fallback-rb2" style="left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5}); width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});"></span>"#,
                right.0,
                top.0,
                self.page_width - right.0,
                self.page_height - top.0,
            ));
        }

        let zero_label = *self.discrete_label_map.get(&Scalar(0.0)).unwrap();
        let mut last_bottom = zero_label;

        // todo: optimize using ds
        let mut max_right_for_row = Vec::<Option<usize>>::new();
        max_right_for_row.resize(self.discrete_value_map.len(), None);
        let mut max_bottom_for_col = Vec::<Option<usize>>::new();
        max_bottom_for_col.resize(self.discrete_value_map.len(), None);

        // Prepend left and top fallbacks
        for post_idx in 1..self.rects.len() {
            let pre_idx = post_idx - 1;
            let (_, rect) = self.rects[pre_idx];

            let (left, top, right, bottom) = self.get_discrete_labels_for_text_item(rect);

            // Prepend whole width blanks
            if top > last_bottom {
                let from = self.discrete_value_map[last_bottom];
                let height = rect.lo.y - from;
                res[pre_idx].0.push_str(&format!(
                    r#"<span class="typst-content-fallback typst-content-fallback-whole" style="left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5}); width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});"></span>"#,
                    0.0,
                    from.0,
                    self.page_width,
                    height.0,
                ));
            }
            last_bottom = last_bottom.max(bottom);

            // Process current item left
            {
                let lefty = &max_right_for_row[top..bottom];
                let mut begin = 0;
                let mut end = 0;

                // group by contiguous same value
                while begin < lefty.len() {
                    while end < lefty.len() && lefty[begin] == lefty[end] {
                        end += 1;
                    }

                    let last_right =
                        lefty[begin].and_then(|v| if v > left { None } else { Some(v) });

                    // if last_right.is_none() && begin+top <=

                    // expand to page border 0.0 if no last right
                    let from = match last_right {
                        Some(last_right) => {
                            (self.discrete_value_map[last_right] + rect.lo.x) / Scalar(2.0)
                        }
                        None => Scalar(0.0),
                    };
                    let width = rect.lo.x - from;

                    let ptop = if last_right.is_none() {
                        (begin + top).max(last_bottom)
                    } else {
                        begin + top
                    };
                    let pbottom = (end + top).min(bottom);

                    if ptop < pbottom {
                        let ptop = self.discrete_value_map[ptop];
                        let pbottom = self.discrete_value_map[pbottom];

                        res[pre_idx].0.push_str(&format!(
                            r#"<span class="typst-content-fallback typst-content-fallback-left" style="left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5}); width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});"></span>"#,
                            from.0,
                            ptop.0,
                            width.0,
                            pbottom.0 - ptop.0,
                        ));
                    }

                    begin = end;
                }

                // maintain max_right_for_row
                for elem in &mut max_right_for_row[top..bottom] {
                    let val = elem.get_or_insert(right);
                    *val = right.max(*val);
                }
            }
        }

        res
    }

    fn get_discrete_labels_for_text_item(&self, rect: Rect) -> (usize, usize, usize, usize) {
        let mut left = *self.discrete_label_map.get(&rect.lo.x).unwrap();
        let mut top = *self.discrete_label_map.get(&rect.lo.y).unwrap();
        let mut right = *self.discrete_label_map.get(&rect.hi.x).unwrap();
        let mut bottom = *self.discrete_label_map.get(&rect.hi.y).unwrap();
        if left > right {
            std::mem::swap(&mut left, &mut right);
        }
        if top > bottom {
            std::mem::swap(&mut top, &mut bottom);
        }

        (left, top, right, bottom)
    }

    fn render_semantics_walk<'a>(
        &mut self,
        ctx: &'a Module,
        ts: tiny_skia::Transform,
        fg: Fingerprint,
        fallbacks: &mut VecDeque<(String, String)>,
        output: &mut Vec<Cow<'a, str>>,
    ) {
        let item = ctx.get_item(&fg).unwrap();

        use VecItem::*;
        match item {
            Group(t) => {
                output.push(Cow::Borrowed(r#"<span class="typst-content-group">"#));
                for (pos, child) in t.0.iter() {
                    let ts = ts.pre_translate(pos.x.0, pos.y.0);
                    self.render_semantics_walk(ctx, ts, *child, fallbacks, output);
                }
                output.push(Cow::Borrowed("</span>"));
            }
            Item(t) => {
                output.push(Cow::Borrowed(r#"<span class="typst-content-group">"#));
                let trans = t.0.clone();
                let trans: ir::Transform = trans.into();
                let ts = ts.pre_concat(trans.into());
                self.render_semantics_walk(ctx, ts, t.1, fallbacks, output);
                output.push(Cow::Borrowed("</span>"));
            }
            Labelled(t) => {
                output.push(Cow::Borrowed(r#""#));
                output.push(Cow::Owned(format!(
                    r#"<span class="typst-content-group" data-typst-label="{}" >"#,
                    escape_str::<AttributeEscapes>(&t.0)
                )));
                self.render_semantics_walk(ctx, ts, t.1, fallbacks, output);
                output.push(Cow::Borrowed("</span>"));
            }
            Text(t) => {
                let text_id = self.dfn_count;
                self.dfn_count += 1;

                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                let can_heavy = self.heavy;
                let size = (t.shape.size) * Scalar(ts.sy);

                let scale_x = t.width().0
                    / (t.content
                        .content
                        .chars()
                        .map(|e| match e.width().unwrap_or_default() {
                            0 => 0.,
                            1 => self.font_metric.semi_char_width,
                            2 => self.font_metric.full_char_width,
                            _ => self.font_metric.emoji_width,
                        })
                        .sum::<f32>()
                        * size.0);

                let (_, rect) = self.rects[text_id];

                let (prepend, append) = fallbacks.pop_front().unwrap();

                if can_heavy {
                    output.push(Cow::Owned(prepend));
                }

                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-text" data-text-id="{}" style="font-size: calc(var(--data-text-height) * {:.5}); line-height: calc(var(--data-text-height) * {:.5}); left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5}); transform: scaleX({:.5})">"#,
                        text_id,
                        size.0,
                        size.0,
                        rect.lo.x.0,
                        rect.lo.y.0,
                        scale_x,
                        // scale_y,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-text" data-text-id="{}" data-matrix="{:.5},{:.5},{:.5},{:.5}" style="font-size: {:.5}px; line-height: calc(var(--data-text-height) * {:.5}); left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5}); transform: scaleX({:.5})">"#,
                        text_id,
                        ts.sx,
                        ts.ky,
                        ts.kx,
                        ts.sy,
                        size.0,
                        size.0,
                        rect.lo.x.0,
                        rect.lo.y.0,
                        scale_x,
                        // scale_y,
                    )));
                }

                output.push(escape::escape_str::<PcDataEscapes>(
                    t.content.content.as_ref(),
                ));
                output.push(Cow::Borrowed("</span>"));

                if can_heavy {
                    output.push(Cow::Owned(append));
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
                            r#" style="font-size: 0px; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});">"#,
                            ts.tx,ts.ty,
                        )));
                    } else {
                        output.push(Cow::Owned(format!(
                            r#" data-matrix="{:.5},{:.5},{:.5},{:.5}" style="font-size: 0px; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});">"#,
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
                        r#" style="font-size: 0px; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});">"#,
                        ts.tx,ts.ty,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{:.5},{:.5},{:.5},{:.5}" style="font-size: 0px; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy,   ts.tx,ts.ty,
                    )));
                }
                let c = c.to_string();
                let c = escape::escape_str::<PcDataEscapes>(&c).into_owned();
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
                        r#" style="font-size: 0px; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});  width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});">"#,
                        ts.tx - 1., ts.ty - 2., t.size.x.0 + 2., t.size.y.0 + 4.,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{:.5},{:.5},{:.5},{:.5}" style="font-size: 0px; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});  width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy, ts.tx,ts.ty, t.size.x.0, t.size.y.0,
                    )));
                }
                output.push(Cow::Borrowed("</a>"));
            }
            // todo: implement in svg
            SizedRawHtml(h) => {
                web_sys::console::log_1(&format!("Html: {}", h.html).into());
                output.push(Cow::Borrowed(r#"<span class="typst-content-html""#));
                let is_regular_scale = ts.sx == 1.0 && ts.sy == 1.0;
                let is_regular_skew = ts.kx == 0.0 && ts.ky == 0.0;
                // todo: correct zindex
                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#" style="zindex: 3; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});  width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});">"#,
                        ts.tx, ts.ty, h.size.x.0, h.size.y.0,
                    )));
                } else {
                    output.push(Cow::Owned(format!(
                        r#" data-matrix="{:.5},{:.5},{:.5},{:.5}" style="zindex: 3; left: calc(var(--data-text-width) * {:.5}); top: calc(var(--data-text-height) * {:.5});  width: calc(var(--data-text-width) * {:.5}); height: calc(var(--data-text-height) * {:.5});">"#,
                        ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty, h.size.x.0, h.size.y.0,
                    )));
                }
                output.push(Cow::Owned(h.html.to_string()));
                output.push(Cow::Borrowed("</span>"));
            }
            Image(..) | Path(..) => {}
            None | ColorTransform(..) | Gradient(..) | Color32(..) | Pattern(..) | Html(..) => {}
        }
    }
}
