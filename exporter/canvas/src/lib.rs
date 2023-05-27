//! Rendering into web_sys::CanvasRenderingContext2d.

#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;

pub(crate) use tiny_skia as sk;

use typst::doc::{Frame, FrameItem, GroupItem, Meta};
use typst::font::{FontFlags, FontInfo, FontVariant};
use typst::geom::Color;
use typst_ts_core::TextContent;
use utils::{js_random64, AbsExt, ToCssExt};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub(crate) mod utils;

pub(crate) mod content;
pub(crate) use content::*;

pub(crate) mod svg;

pub(crate) mod render_image;
pub(crate) mod render_shape;
pub(crate) mod render_text;

pub struct CanvasRenderTask<'a> {
    canvas: &'a CanvasRenderingContext2d,

    pixel_per_pt: f32,
    fill: Color,

    width_px: u32,
    height_px: u32,
    raw_height: f32,

    session_id: String,

    pub content: TextContent,

    font_map: HashMap<FontInfo, u32>,
}

pub type LigatureMap = std::collections::HashMap<
    (String, FontVariant, FontFlags),
    std::collections::HashMap<u16, std::string::String>,
>;

impl<'a> CanvasRenderTask<'a> {
    pub fn new(
        canvas: &'a CanvasRenderingContext2d,
        doc: &'a typst::doc::Document,
        ligature_map: &'a LigatureMap,
        page_off: usize,
        pixel_per_pt: f32,
        fill: Color,
    ) -> Result<Self, JsValue> {
        if pixel_per_pt <= 0. {
            panic!("pixel_per_pt must be greater than 0");
        }

        let (width_px, height_px) = {
            let size = doc.pages[page_off].size();
            let width_px = (pixel_per_pt * (size.x.to_pt() as f32)).round().max(1.0) as u32;
            let height_px = (pixel_per_pt * (size.y.to_pt() as f32)).round().max(1.0) as u32;

            (width_px, height_px)
        };

        let session_id = format!("{:x}", js_random64());

        let canvas_ref = canvas.canvas().unwrap();
        canvas_ref.set_attribute("data-typst-session", &session_id)?;

        Ok(Self {
            canvas,

            pixel_per_pt,
            fill,

            width_px,
            height_px,
            raw_height: height_px as f32 / pixel_per_pt,

            session_id,

            content: TextContent::default(),
            font_map: HashMap::default(),
        })
    }

    #[inline]
    fn sync_transform(&mut self, transform: sk::Transform) {
        // [ a c e ]
        // [ b d f ]
        // [ 0 0 1 ]

        // horizontal scaling
        let a = transform.sx as f64;
        // horizontal skewing
        let b = transform.ky as f64;
        // vertical skewing
        let c = transform.kx as f64;
        // vertical scaling
        let d = transform.sy as f64;
        // horizontal moving
        let e = transform.tx as f64;
        // vertical moving
        let f = transform.ty as f64;

        self.canvas.transform(a, b, c, d, e, f).unwrap();
    }

    /// Directly render a frame into the canvas.
    pub fn render(&mut self, frame: &Frame) {
        self.canvas.set_fill_style(&self.fill.to_css().into());
        self.canvas
            .fill_rect(0., 0., self.width_px as f64, self.height_px as f64);

        let ts = sk::Transform::from_scale(self.pixel_per_pt, self.pixel_per_pt);
        self.render_frame(ts, None, frame);
    }

    /// Render a frame into the canvas.
    fn render_frame(&mut self, ts: sk::Transform, mask: Option<&sk::Mask>, frame: &Frame) {
        let mut text_flow = TextFlow::new();

        for (pos, item) in frame.items() {
            let x = pos.x.to_f32();
            let y = pos.y.to_f32();
            let ts = ts.pre_translate(x, y);

            match item {
                FrameItem::Group(group) => {
                    self.render_group(ts, mask, group);
                }
                FrameItem::Text(text) => {
                    let (next_text_flow, has_eol) = TextFlow::notify(text_flow, &ts, text);
                    text_flow = next_text_flow;

                    // has end of line (concept from pdf.js)
                    if has_eol {
                        self.append_text_break(ts, text)
                    }

                    self.render_text(ts, mask, text);
                }
                FrameItem::Shape(shape, _) => {
                    self.render_shape(ts, mask, shape);
                }
                FrameItem::Image(image, size, _) => {
                    self.render_image(ts, mask, image, *size);
                }
                FrameItem::Meta(meta, _) => match meta {
                    Meta::Link(_) => {}
                    Meta::Elem(_) => {}
                    Meta::PageNumbering(_) => {}
                    Meta::Hide => {}
                },
            }
        }
    }

    /// Render a group frame with optional transform and clipping into the canvas.
    fn render_group(&mut self, ts: sk::Transform, mask: Option<&sk::Mask>, group: &GroupItem) {
        let ts = ts.pre_concat(group.transform.into());

        let mut mask = mask;

        let storage;
        if group.clips {
            let size = group.frame.size();
            let w = size.x.to_f32();
            let h = size.y.to_f32();
            if let Some(path) = sk::Rect::from_xywh(0.0, 0.0, w, h)
                .map(sk::PathBuilder::from_rect)
                .and_then(|path| path.transform(ts))
            {
                if let Some(mask) = mask {
                    let mut mask = mask.clone();
                    mask.intersect_path(
                        &path,
                        sk::FillRule::default(),
                        false,
                        sk::Transform::default(),
                    );
                    storage = mask;
                } else {
                    let pxw = self.width_px;
                    let pxh = self.height_px;
                    let Some(mut mask) = sk::Mask::new(pxw, pxh) else {
                        // Fails if clipping rect is empty. In that case we just
                        // clip everything by returning.
                        return;
                    };

                    mask.fill_path(
                        &path,
                        sk::FillRule::default(),
                        false,
                        sk::Transform::default(),
                    );
                    storage = mask;
                };

                mask = Some(&storage);
            }
        }

        self.render_frame(ts, mask, &group.frame);
    }
}
