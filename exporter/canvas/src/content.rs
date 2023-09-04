use std::collections::HashMap;
use tiny_skia as sk;
use typst::{
    font::FontInfo,
    geom::{Axis, Dir},
};

use typst_ts_core::{
    hash::Fingerprint,
    vector::{
        flat_ir::{self, FlatSvgItem, FlatTextItem, GroupRef, Module},
        flat_vm::{FlatGroupContext, FlatRenderVm},
        ir::{self, Abs, Axes, FontIndice, FontRef, Ratio, Scalar, SvgItem},
        vm::{GroupContext, RenderVm, TransformContext},
    },
    TextContent,
};

pub struct TextContentBuilder {
    ts: sk::Transform,
}

impl From<TextContentBuilder> for () {
    fn from(_: TextContentBuilder) -> Self {}
}

/// See [`TransformContext`].
impl<C> TransformContext<C> for TextContentBuilder {
    fn transform_matrix(mut self, _ctx: &mut C, m: &ir::Transform) -> Self {
        let sub_ts: sk::Transform = (*m).into();
        self.ts = self.ts.post_concat(sub_ts);
        self
    }

    fn transform_translate(mut self, _ctx: &mut C, matrix: Axes<Abs>) -> Self {
        self.ts = self.ts.post_translate(matrix.x.0, matrix.y.0);
        self
    }

    fn transform_scale(mut self, _ctx: &mut C, x: Ratio, y: Ratio) -> Self {
        self.ts = self.ts.post_scale(x.0, y.0);
        self
    }

    fn transform_rotate(self, _ctx: &mut C, _matrix: Scalar) -> Self {
        todo!()
    }

    fn transform_skew(mut self, _ctx: &mut C, matrix: (Ratio, Ratio)) -> Self {
        self.ts = self.ts.post_concat(sk::Transform {
            sx: 1.,
            sy: 1.,
            kx: matrix.0 .0,
            ky: matrix.1 .0,
            tx: 0.,
            ty: 0.,
        });
        self
    }

    fn transform_clip(self, _ctx: &mut C, _matrix: &ir::PathItem) -> Self {
        self
    }
}

trait TranslateCtx {
    fn translate(&mut self, x: Scalar, y: Scalar);
}

/// See [`GroupContext`].
impl<C: TranslateCtx + RenderVm<Resultant = ()>> GroupContext<C> for TextContentBuilder {
    fn render_item_at(&mut self, ctx: &mut C, pos: ir::Point, item: &ir::SvgItem) {
        ctx.translate(pos.x, pos.y);
        ctx.render_item(item);
        ctx.translate(-pos.x, -pos.y);
    }
}

/// See [`FlatGroupContext`].
impl<'m, C: TranslateCtx + FlatRenderVm<'m, Resultant = ()>> FlatGroupContext<C>
    for TextContentBuilder
{
    fn render_item_ref_at(&mut self, ctx: &mut C, pos: ir::Point, item: &Fingerprint) {
        ctx.translate(pos.x, pos.y);
        ctx.render_flat_item(item);
        ctx.translate(-pos.x, -pos.y);
    }
}

/// Task to create text content with vector IR
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub struct TextContentTask<'m, 't> {
    pub module: &'m Module,

    ts: sk::Transform,
    pub text_content: &'t mut TextContent,
    pub page_height: f32,

    font_map: HashMap<FontInfo, u32>,
    flat_font_map: HashMap<FontRef, u32>,
}

// todo: ugly implementation
impl<'m, 't> TextContentTask<'m, 't> {
    pub fn new(module: &'m Module, text_content: &'t mut TextContent) -> Self {
        Self {
            module,
            ts: sk::Transform::identity(),
            text_content,
            page_height: 0.,
            font_map: HashMap::new(),
            flat_font_map: HashMap::new(),
        }
    }

    pub fn process_item(&mut self, ts: sk::Transform, item: &ir::SvgItem) {
        match item {
            SvgItem::Transformed(t) => self.process_item(
                ts.pre_concat({
                    let t: typst_ts_core::vector::geom::Transform = t.0.clone().into();
                    t.into()
                }),
                &t.1,
            ),
            SvgItem::Group(group) => self.process_group(ts, group),
            SvgItem::Text(text) => self.process_text(ts, text),
            _ => {}
        }
    }

    fn process_group(&mut self, ts: sk::Transform, group: &ir::GroupItem) {
        let mut text_flow = TextFlow::new();

        for (pos, item) in &group.0 {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);

            match item {
                SvgItem::Transformed(t) => self.process_item(
                    ts.pre_concat({
                        let t: typst_ts_core::vector::geom::Transform = t.0.clone().into();
                        t.into()
                    }),
                    &t.1,
                ),
                SvgItem::Group(group) => {
                    self.process_group(ts, group);
                }
                SvgItem::Text(text) => {
                    let (next_text_flow, has_eol) = TextFlow::notify(text_flow, &ts, &text.shape);
                    text_flow = next_text_flow;

                    // has end of line (concept from pdf.js)
                    if has_eol {
                        let font_name = self.append_text_font(&text.font);
                        self.append_text_break(ts, font_name, &text.shape)
                    }

                    self.process_text(ts, text);
                }
                _ => {}
            }
        }
    }

    fn process_text(&mut self, ts: sk::Transform, text: &ir::TextItem) {
        let font_name = self.append_text_font(&text.font);
        let width = text.content.glyphs.iter().map(|g| g.0 .0 + g.1 .0).sum();
        self.append_text_content(
            ts,
            text.content.content.as_ref().to_string(),
            font_name,
            width,
            text.shape.size.0,
            &text.shape,
            false,
        )
    }

    pub fn process_flat_item(&mut self, ts: sk::Transform, item: &Fingerprint) {
        let item = self.module.get_item(item).unwrap();
        match item {
            FlatSvgItem::Item(t) => self.process_flat_item(
                ts.pre_concat({
                    let t: typst_ts_core::vector::geom::Transform = t.0.clone().into();
                    t.into()
                }),
                &t.1,
            ),
            FlatSvgItem::Group(group) => self.process_flat_group(ts, group),
            FlatSvgItem::Text(text) => self.process_flat_text(ts, text),
            _ => {}
        }
    }

    fn process_flat_group(&mut self, ts: sk::Transform, group: &GroupRef) {
        let mut text_flow = TextFlow::new();

        for (pos, item) in group.0.as_ref() {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);

            let item = self.module.get_item(item).unwrap();
            match item {
                FlatSvgItem::Item(t) => self.process_flat_item(
                    ts.pre_concat({
                        let t: typst_ts_core::vector::geom::Transform = t.0.clone().into();
                        t.into()
                    }),
                    &t.1,
                ),
                FlatSvgItem::Group(group) => {
                    self.process_flat_group(ts, group);
                }
                FlatSvgItem::Text(text) => {
                    let (next_text_flow, has_eol) = TextFlow::notify(text_flow, &ts, &text.shape);
                    text_flow = next_text_flow;

                    // has end of line (concept from pdf.js)
                    if has_eol {
                        let font_name = self.append_flat_text_font(text.font.clone());
                        self.append_text_break(ts, font_name, &text.shape)
                    }

                    self.process_flat_text(ts, text);
                }
                _ => {}
            }
        }
    }

    fn process_flat_text(&mut self, ts: sk::Transform, text: &FlatTextItem) {
        let font_name = self.append_flat_text_font(text.font.clone());
        let width = text.content.glyphs.iter().map(|g| g.0 .0 + g.1 .0).sum();
        self.append_text_content(
            ts,
            text.content.content.as_ref().to_string(),
            font_name,
            width,
            text.shape.size.0,
            &text.shape,
            false,
        )
    }

    fn append_flat_text_font(&mut self, font: FontRef) -> u32 {
        if let Some(&font) = self.flat_font_map.get(&font) {
            return font;
        }

        if self.text_content.styles.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let font_item = &self.module.fonts[font.idx as usize];

        let font_ref = self.text_content.styles.len() as u32;
        self.flat_font_map.insert(font, font_ref);

        self.text_content
            .styles
            .push(typst_ts_core::content::TextStyle {
                font_family: font_item.family.as_ref().to_owned(),
                ascent: font_item.ascender.0,
                descent: font_item.descender.0,
                vertical: font_item.vertical,
            });
        font_ref
    }

    // todo: unify with append_flat_text_font
    fn append_text_font(&mut self, font: &typst::font::Font) -> u32 {
        if let Some(&font) = self.font_map.get(font.info()) {
            return font;
        }

        if self.text_content.styles.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let font_ref = self.text_content.styles.len() as u32;
        self.font_map.insert(font.info().clone(), font_ref);
        self.text_content
            .styles
            .push(typst_ts_core::content::TextStyle {
                font_family: font.info().family.clone(),
                ascent: font.metrics().ascender.get() as f32,
                descent: font.metrics().descender.get() as f32,
                vertical: false,
            });
        font_ref
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn append_text_content(
        &mut self,
        ts: sk::Transform,
        text_content: String,
        font_name: u32,
        width: f32,
        height: f32,
        shape: &ir::TextShape,
        has_eol: bool,
    ) {
        // adapt scale for pdf.js

        self.text_content
            .items
            .push(typst_ts_core::content::TextItem {
                str: text_content,
                // todo: real direction of the text
                dir: shape.dir.as_ref().to_owned(),
                // todo: we should set the original height, not specially for pdf.js
                width,
                height,
                transform: [
                    shape.size.0,
                    ts.ky,
                    ts.kx,
                    shape.size.0,
                    ts.tx,
                    self.page_height - ts.ty,
                ],
                font_name,
                has_eol,
            });
    }

    pub(crate) fn append_text_break(
        &mut self,
        ts: sk::Transform,
        font_name: u32,
        shape: &ir::TextShape,
    ) {
        self.append_text_content(ts, "".to_string(), font_name, 0., 0., shape, true)
    }
}

impl<'m, 't> TranslateCtx for TextContentTask<'m, 't> {
    fn translate(&mut self, x: Scalar, y: Scalar) {
        self.ts = self.ts.post_translate(x.0, y.0);
    }
}

pub struct TextFlow {
    dir: Dir,
    tx: f32,
    ty: f32,
    last_diff: Option<f32>,
}

impl TextFlow {
    pub fn new() -> Option<Self> {
        None
    }

    pub fn notify(
        mut this: Option<Self>,
        ts: &sk::Transform,
        shape: &ir::TextShape,
    ) -> (Option<Self>, bool) {
        let dir = match shape.dir.as_ref() {
            "ltr" => Dir::LTR,
            "rtl" => Dir::RTL,
            "ttb" => Dir::TTB,
            "btt" => Dir::BTT,
            _ => unreachable!(),
        };
        let advance_flow = |last_diff: Option<f32>| {
            Some(TextFlow {
                dir,
                tx: ts.tx,
                ty: ts.ty,
                last_diff,
            })
        };

        let mut has_eol = false;
        if let Some(TextFlow {
            dir: prev_dir,
            tx,
            ty,
            last_diff,
        }) = this
        {
            if prev_dir != dir {
                this = advance_flow(None);
                has_eol = true;
            } else {
                match dir.axis() {
                    Axis::X => {
                        if ts.ty != ty {
                            let diff = ts.ty - ty;
                            this = advance_flow(Some(diff));
                            has_eol = if let Some(last_diff) = last_diff {
                                last_diff != diff || ts.tx != tx
                            } else {
                                false
                            };
                        }
                    }
                    Axis::Y => {
                        if ts.tx != tx {
                            let diff = ts.tx - tx;
                            this = advance_flow(Some(diff));
                            has_eol = if let Some(last_diff) = last_diff {
                                last_diff != diff || ts.ty != ty
                            } else {
                                false
                            };
                        }
                    }
                }
            }
        } else {
            this = advance_flow(None);
        }

        (this, has_eol)
    }
}

impl<'m, 't> FontIndice<'m> for TextContentTask<'m, 't> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize)
    }
}

impl<'m, 't> RenderVm for TextContentTask<'m, 't> {
    type Resultant = ();
    type Group = TextContentBuilder;

    fn start_group(&mut self) -> Self::Group {
        Self::Group { ts: self.ts }
    }
}

impl<'m, 't> FlatRenderVm<'m> for TextContentTask<'m, 't> {
    type Resultant = ();
    type Group = TextContentBuilder;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m flat_ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&mut self, _v: &Fingerprint) -> Self::Group {
        self.start_group()
    }
}
