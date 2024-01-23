use std::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque},
};

use reflexo_vec2canvas::CanvasStateGuard;
use typst_ts_core::hash::Fingerprint;
use typst_ts_svg_exporter::{
    ir::{self, Point, Rect, Scalar, VecItem},
    Module,
};
use unicode_width::UnicodeWidthChar;
use web_sys::{wasm_bindgen::JsCast, HtmlCanvasElement};

use crate::escape::{self, AttributeEscapes, PcDataEscapes};

#[derive(Clone, Copy)]
pub struct BrowserFontMetric {
    semi_char_width: f32,
    full_char_width: f32,
    emoji_width: f32,
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
        let semi_char_width = metrics.width();
        let metrics = ctx.measure_text("喵").unwrap();
        let full_char_width = metrics.width();
        let metrics = ctx.measure_text("🦄").unwrap();
        let emoji_width = metrics.width();
        // let a_height =
        //     (metrics.font_bounding_box_descent() +
        // metrics.font_bounding_box_ascent()).abs();

        Self {
            semi_char_width: (semi_char_width / 128.) as f32,
            full_char_width: (full_char_width / 128.) as f32,
            emoji_width: (emoji_width / 128.) as f32,
            // height: (a_height / 128.) as f32,
        }
    }
}

pub struct SemanticsBackend {
    heavy: bool,
    font_metric: BrowserFontMetric,
    page_width: f32,
    dfn_count: usize,
    text_rects: Vec<(Fingerprint, Rect)>,
    discrete_label_map: BTreeMap<Scalar, usize>,
    discrete_value_map: Vec<Scalar>,
}

impl SemanticsBackend {
    pub fn new(heavy: bool, font_metric: BrowserFontMetric, width: f32) -> Self {
        SemanticsBackend {
            heavy,
            font_metric,
            page_width: width,
            dfn_count: 0,
            text_rects: vec![],
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
        let mut fallbacks = self.calc_text_item_fallbacks(ctx);
        self.dfn_count = 0;
        self.render_semantics_walk(ctx, ts, fg, &mut fallbacks, output);
    }

    fn prepare_text_rects<'a>(
        &mut self,
        ctx: &'a Module,
        ts: tiny_skia::Transform,
        fg: Fingerprint,
    ) {
        let item = ctx.get_item(&fg).unwrap();
        use VecItem::*;
        match item {
            Group(t, _) => {
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

                self.text_rects.push((
                    fg,
                    Rect {
                        lo: Point { x: tx, y: ty },
                        hi: Point { x: tx2, y: ty2 },
                    },
                ));
            }
            _ => {}
        }
    }

    fn prepare_discrete_map<'a>(&mut self) {
        let nums = &mut self.discrete_value_map;

        for (_, rect) in self.text_rects.iter() {
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
            const EPS: f32 = 0.5;
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
    fn calc_text_item_fallbacks<'a>(&mut self, ctx: &'a Module) -> VecDeque<(String, String)> {
        let mut res = VecDeque::new();
        res.resize(self.text_rects.len(), (String::new(), String::new()));

        // Map<row, Vec<(left, right, idx)>>
        let mut row_idxs = vec![self.discrete_label_map[&Scalar(0.0)]];
        let mut row_items: BTreeMap<usize, Vec<(usize, usize, usize)>> = Default::default();

        // init all row indexes
        for (_, rect) in self.text_rects.iter() {
            let (_, top, _, bottom) = self.get_discrete_labels_for_text_item(*rect);

            row_idxs.push(top);
            row_idxs.push(bottom);
        }

        row_idxs.sort();
        row_idxs.dedup();

        for idx in &row_idxs {
            row_items.entry(*idx).or_insert(Default::default());
        }

        // todo: lazy tag 2d segment tree for optimization
        for (idx, (_, rect)) in self.text_rects.iter().enumerate() {
            let (left, top, right, bottom) = self.get_discrete_labels_for_text_item(*rect);

            let rng = if top <= bottom {
                top..bottom
            } else {
                bottom..top
            };

            row_items.range_mut(rng).for_each(|(_, v)| {
                v.push((left, right, idx));
            });
        }

        // zip iter for pairwise
        let it = row_items.iter_mut().zip(row_idxs.iter().skip(1));
        let mut last_blank = Option::<Scalar>::None;

        for ((row, items), nrow) in it {
            let top = self.discrete_value_map[*row];
            let bottom = self.discrete_value_map[*nrow];

            items.sort();

            web_sys::console::log_1(
                &format!("processing row: {} ~ {} with {:?}", top.0, bottom.0, items).into(),
            );

            if items.is_empty() {
                // insert a whole page width bar
                // but delay to the next row with at least one item
                last_blank.get_or_insert(top);
            } else {
                // process last blank
                if let Some(blank_top) = last_blank {
                    let blank_left = Scalar(0.0);
                    let blank_height = top - blank_top;

                    res[items[0].2].0.push_str(&format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        blank_left.0,
                        blank_top.0,
                        self.page_width,
                        blank_height.0,
                    ));

                    last_blank = None;
                }

                // merge overlap items: (left, right, first_idx, last_idx)
                let mut merged_items: Vec<(usize, usize, usize, usize)> = vec![];
                let mut last_item = (items[0].0, items[0].1, items[0].2, items[0].2);
                for item in items.iter().skip(1) {
                    if last_item.1 >= item.0 {
                        last_item.1 = item.1.max(last_item.1);
                        last_item.3 = item.2.max(last_item.3);
                    } else {
                        merged_items.push(last_item);
                        last_item = (item.0, item.1, item.2, item.2);
                    }
                }
                merged_items.push(last_item);

                // insert fallbacks for last right
                if let Some((_, d_right, _, last_idx)) = merged_items.last() {
                    let right = self.discrete_value_map[*d_right];
                    res[*last_idx].1.push_str(&format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        right.0,
                        top.0,
                        self.page_width - right.0,
                        bottom.0 - top.0,
                    ));
                }

                // insert fallbacks for left
                if let Some((d_left, _, first_idx, _)) = merged_items.first() {
                    let left = self.discrete_value_map[*d_left];
                    res[*first_idx].0.push_str(&format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        0.0,
                        top.0,
                        left.0,
                        bottom.0 - top.0,
                    ));
                }

                // insert fallbacks for middle
                for wind in merged_items.windows(2) {
                    let prior = wind[0];
                    let post = wind[1];
                    let (_, d_prior_right, _, piror_idx) = prior;
                    let (d_post_left, _, post_idx, _) = post;

                    let prior_end = self.discrete_value_map[d_prior_right];
                    let post_begin = self.discrete_value_map[d_post_left];
                    let width = (post_begin - prior_end) / Scalar(2.0);

                    res[piror_idx].1.push_str(&format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        prior_end.0,
                        top.0,
                        width.0,
                        bottom.0 - top.0,
                    ));

                    res[post_idx].0.push_str(&format!(
                        r#"<span class="typst-content-fallback" style="left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); width: calc(var(--data-text-width) * {}); height: calc(var(--data-text-height) * {});"></span>"#,
                        prior_end.0 + width.0,
                        top.0,
                        width.0,
                        bottom.0 - top.0,
                    ));
                }
            }
        }

        res
    }

    fn get_discrete_labels_for_text_item(&self, rect: Rect) -> (usize, usize, usize, usize) {
        let left = self.discrete_label_map.get(&rect.lo.x).unwrap();
        let top = self.discrete_label_map.get(&rect.lo.y).unwrap();
        let right = self.discrete_label_map.get(&rect.hi.x).unwrap();
        let bottom = self.discrete_label_map.get(&rect.hi.y).unwrap();
        (*left, *top, *right, *bottom)
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
            Group(t, _) => {
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

                let (_, rect) = self.text_rects[text_id];

                let (prepend, append) = fallbacks.pop_front().unwrap();

                if can_heavy {
                    output.push(Cow::Borrowed(r#"<!-- This is prepend -->"#));
                    output.push(Cow::Owned(prepend));
                    output.push(Cow::Borrowed(r#"<!-- Prepend end -->"#));
                }

                if is_regular_scale && is_regular_skew {
                    output.push(Cow::Owned(format!(
                        r#"<span class="typst-content-text" style="font-size: calc(var(--data-text-height) * {}); line-height: calc(var(--data-text-height) * {}); left: calc(var(--data-text-width) * {}); top: calc(var(--data-text-height) * {}); transform: scaleX({})">"#,
                        size.0,
                        size.0,
                        rect.lo.x.0,
                        rect.lo.y.0,
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
                    output.push(Cow::Borrowed(r#"<!-- This is append -->"#));
                    output.push(Cow::Owned(append));
                    output.push(Cow::Borrowed(r#"<!-- Append end -->"#));
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
