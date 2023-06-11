#![allow(dead_code)]

use typst::{
    doc::TextItem,
    geom::{Axis, Dir},
};

use super::SvgRenderTask;
use crate::{sk, utils::AbsExt, RenderFeature};

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

    pub fn notify(mut this: Option<Self>, ts: &sk::Transform, dir: Dir) -> (Option<Self>, bool) {
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

impl<'m, 't, Feat: RenderFeature> SvgRenderTask<'m, 't, Feat> {
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
                vertical: false, // todo: check vertical
            });
        font_ref
    }

    pub(crate) fn append_text_content(
        &mut self,
        ts: sk::Transform,
        text: &TextItem,
        text_content: String,
        width: f32,
        height: f32,
        has_eol: bool,
    ) {
        // adapt scale for pdf.js

        let font_name = self.append_text_font(&text.font);
        self.text_content
            .items
            .push(typst_ts_core::content::TextItem {
                str: text_content,
                // todo: real direction of the text
                dir: match text.lang.dir() {
                    Dir::LTR => "ltr".to_string(),
                    Dir::RTL => "rtl".to_string(),
                    Dir::TTB => "ttb".to_string(),
                    Dir::BTT => "btt".to_string(),
                },
                // todo: we should set the original height, not specially for pdf.js
                width,
                height,
                transform: [
                    text.size.to_f32(),
                    ts.ky,
                    ts.kx,
                    text.size.to_f32(),
                    ts.tx,
                    self.raw_height - ts.ty,
                ],
                font_name,
                has_eol,
            });
    }

    pub(crate) fn append_text_break(&mut self, ts: sk::Transform, text: &TextItem) {
        self.append_text_content(ts, text, "".to_string(), 0., 0., true)
    }
}
