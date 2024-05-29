use std::collections::HashMap;
use tiny_skia as sk;

use reflexo::{
    content::{self, TextContent},
    hash::Fingerprint,
    vector::{
        ir::{
            self, Abs, Axes, FontIndice, FontRef, GroupRef, Module, Ratio, Scalar, TextItem,
            Transform, VecItem,
        },
        vm::{GroupContext, RenderVm, TransformContext},
    },
};

/// Builds text content with vector IR
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

/// See [`FlatGroupContext`].
impl<'m, C: TranslateCtx + RenderVm<'m, Resultant = ()>> GroupContext<C> for TextContentBuilder {
    fn render_item_at(&mut self, ctx: &mut C, pos: ir::Point, item: &Fingerprint) {
        ctx.translate(pos.x, pos.y);
        ctx.render_item(item);
        ctx.translate(-pos.x, -pos.y);
    }
}

/// Task to create text content with vector IR
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub struct TextContentTask<'m, 't> {
    /// The module which stores the item data
    pub module: &'m Module,
    /// Sets a page height so that we can calculate the position of the text
    pub page_height: f32,
    /// The resultant, a list of text content
    pub text_content: &'t mut TextContent,

    ts: sk::Transform,

    flat_font_map: HashMap<FontRef, u32>,
}

// todo: ugly implementation
impl<'m, 't> TextContentTask<'m, 't> {
    /// Creates a new task
    pub fn new(module: &'m Module, text_content: &'t mut TextContent) -> Self {
        Self {
            module,
            ts: sk::Transform::identity(),
            text_content,
            page_height: 0.,
            flat_font_map: HashMap::new(),
        }
    }

    /// Collects text content in a vector item
    pub fn process_flat_item(&mut self, ts: sk::Transform, item: &Fingerprint) {
        let item = self.module.get_item(item).unwrap();
        match item {
            VecItem::Item(t) => self.process_flat_item(
                ts.pre_concat({
                    let t: Transform = t.0.clone().into();
                    t.into()
                }),
                &t.1,
            ),
            VecItem::Group(group) => self.process_flat_group(ts, group),
            VecItem::Text(text) => self.process_flat_text(ts, text),
            _ => {}
        }
    }

    fn process_flat_group(&mut self, ts: sk::Transform, group: &GroupRef) {
        let mut text_flow = TextFlow::new();

        for (pos, item) in group.0.as_ref() {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);

            let item = self.module.get_item(item).unwrap();
            match item {
                VecItem::Item(t) => self.process_flat_item(
                    ts.pre_concat({
                        let t: Transform = t.0.clone().into();
                        t.into()
                    }),
                    &t.1,
                ),
                VecItem::Group(group) => {
                    self.process_flat_group(ts, group);
                }
                VecItem::Text(text) => {
                    let (next_text_flow, has_eol) = TextFlow::notify(text_flow, &ts, &text.shape);
                    text_flow = next_text_flow;

                    // has end of line (concept from pdf.js)
                    if has_eol {
                        let font_name = self.append_flat_text_font(text.shape.font);
                        self.append_text_break(ts, font_name, &text.shape)
                    }

                    self.process_flat_text(ts, text);
                }
                _ => {}
            }
        }
    }

    fn process_flat_text(&mut self, ts: sk::Transform, text: &TextItem) {
        let font_name = self.append_flat_text_font(text.shape.font);
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

        let font_item = self.module.get_font(&font).unwrap();

        let font_ref = self.text_content.styles.len() as u32;
        self.flat_font_map.insert(font, font_ref);

        self.text_content.styles.push(content::TextStyle {
            font_family: font_item.family.as_ref().to_owned(),
            ascent: font_item.ascender.0,
            descent: font_item.descender.0,
            vertical: font_item.vertical,
        });
        font_ref
    }

    #[allow(clippy::too_many_arguments)]
    fn append_text_content(
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

        self.text_content.items.push(content::TextItem {
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

    fn append_text_break(&mut self, ts: sk::Transform, font_name: u32, shape: &ir::TextShape) {
        self.append_text_content(ts, "".to_string(), font_name, 0., 0., shape, true)
    }
}

impl<'m, 't> TranslateCtx for TextContentTask<'m, 't> {
    fn translate(&mut self, x: Scalar, y: Scalar) {
        self.ts = self.ts.post_translate(x.0, y.0);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

struct TextFlow {
    dir: Axis,
    tx: f32,
    ty: f32,
    last_diff: Option<f32>,
}

impl TextFlow {
    fn new() -> Option<Self> {
        None
    }

    fn notify(
        mut this: Option<Self>,
        ts: &sk::Transform,
        shape: &ir::TextShape,
    ) -> (Option<Self>, bool) {
        let dir = match shape.dir.as_ref() {
            "ltr" | "rtl" => Axis::X,
            "ttb" | "btt" => Axis::Y,
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
                match dir {
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

impl<'m, 't> RenderVm<'m> for TextContentTask<'m, 't> {
    type Resultant = ();
    type Group = TextContentBuilder;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m ir::VecItem> {
        self.module.get_item(value)
    }

    fn start_group(&mut self, _v: &Fingerprint) -> Self::Group {
        Self::Group { ts: self.ts }
    }
}
