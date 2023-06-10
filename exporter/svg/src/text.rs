use ttf_parser::GlyphId;
use typst::font::Font;
use typst_ts_core::{error::prelude::ZResult, font::GlyphProvider};

use crate::{
    ir::{FlatTextItem, GlyphItem, TextShape},
    utils::{console_log, AbsExt},
    RenderFeature, SvgRenderTask,
};

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Render a text run into the self.canvas.
    pub(crate) fn render_text(&mut self, text: &FlatTextItem) -> ZResult<String> {
        let _r = self.perf_event("render_text");

        let mut text_list = vec![];
        let shape = &text.shape;
        text_list.push(format!(
            r#"<g transform="scale({},{})">"#,
            shape.ppem, -shape.ppem
        ));

        //  todo: fill
        let mut x = 0f32;
        for (idx, glyph) in text.glyphs.iter().enumerate() {
            let glyph = self.module.get_glyph(*glyph).unwrap();
            let t = text.step[idx];

            let offset = x + t.0.to_f32();
            let ts = offset / shape.ppem;

            match glyph {
                GlyphItem::Raw(font, id) => {
                    let font = font.clone();
                    let id = *id;
                    // todo: server side render
                    let e = self
                        .render_svg_glyph(ts, shape.ppem, &font, id)
                        .or_else(|| self.render_bitmap_glyph(shape.ppem, &font, id))
                        .or_else(|| self.render_outline_glyph(ts, shape, &font, id));
                    if let Some(e) = e {
                        text_list.push(e);
                    }
                }
            }

            x += t.1.to_f32();
        }
        text_list.push("</g>".to_string());

        Ok(text_list.join(""))
    }

    /// Render an SVG glyph into the self.canvas.
    /// More information: https://learn.microsoft.com/zh-cn/typography/opentype/spec/svg
    fn render_svg_glyph(
        &mut self,
        _ts: f32,
        _text_size: f32,
        _font: &Font,
        _id: GlyphId,
    ) -> Option<String> {
        // let _r = self.perf_event("render_svg_glyph");
        // let glyph_image = extract_svg_glyph(self.glyph_provider.clone(), font, id)?;

        // // position our image
        // let ascender = font.metrics().ascender.at(text_size).to_f32();

        // let size = text_size.to_f32();
        // // make ascender back
        // let ts = ts.pre_translate(0., -ascender);
        // // pre scale to correct size
        // let ts = ts.pre_scale(
        //     size / glyph_image.width() as f32,
        //     size / glyph_image.height() as f32,
        // );

        // self.render_image(
        //     ts,
        //     &glyph_image,
        //     Size::new(
        //         Abs::pt(glyph_image.width() as f64),
        //         Abs::pt(glyph_image.height() as f64),
        //     ),
        // )
        // .ok()
        // panic!("not support svg glyph")
        None
    }

    /// Render a bitmap glyph into the self.canvas.
    fn render_bitmap_glyph(
        &mut self,
        _text_size: f32,
        _font: &Font,
        _id: GlyphId,
    ) -> Option<String> {
        // let _r = self.perf_event("render_bitmap_glyph");
        // let size = text_size.to_f32();
        // let ppem = (size * ts.sy) as u16;

        // let (glyph_image, raster_x, raster_y) =
        //     extract_bitmap_glyph(self.glyph_provider.clone(), font, id, ppem)?;

        // // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // // and maybe also for Noto Color Emoji. And: Is the size calculation
        // // correct?
        // let h = text_size;
        // let w = (glyph_image.width() as f64 / glyph_image.height() as f64) * h;

        // let dx = (raster_x as f32) / (glyph_image.width() as f32) * size;
        // let dy = (raster_y as f32) / (glyph_image.height() as f32) * size;
        // let ts = ts.pre_translate(dx, -size - dy);

        // self.render_image(&glyph_image, Size::new(w, h)).ok()
        // panic!("not support bitmap glyph")
        None
    }

    /// Render an outline glyph into the canvas. This is the "normal" case.
    fn render_outline_glyph(
        &mut self,
        x: f32,
        shape: &TextShape,
        font: &Font,
        id: GlyphId,
    ) -> Option<String> {
        let _r = self.perf_event("render_outline_glyph");

        // Scale is in pixel per em, but curve data is in font design units, so
        // we have to divide by units per em.

        if cfg!(feature = "debug_glyph_render") {
            console_log!("render_outline_glyph: {:?}", font.info());
        }

        // todo: error handling, reuse color

        let glyph_data = extract_outline_glyph(self.glyph_provider.clone(), font, id)?;
        let glyph_id;
        if let Some(idx) = self.glyph_defs.get(&glyph_data) {
            glyph_id = idx.1;
        } else {
            let new_id = self.glyph_defs.len() as u32;
            let symbol_def = format!(
                r#"<symbol overflow="visible" id="g{:x}">{}</symbol>"#,
                new_id, glyph_data
            );
            self.glyph_defs.insert(glyph_data, (symbol_def, new_id));
            glyph_id = new_id;
        }

        let fill = if shape.fill.as_ref() == "rgba(0, 0, 0, 255)" {
            r#" fill="black" "#.to_owned()
        } else {
            format!(r#" fill="{}" "#, shape.fill)
        };

        Some(format!(
            r##"<use{}href="#g{:x}" x="{:.1}"/>"##,
            fill, glyph_id, x
        ))
    }
}

#[comemo::memoize]
fn extract_outline_glyph(g: GlyphProvider, font: &Font, id: GlyphId) -> Option<String> {
    let d = g.outline_glyph(font, id)?;
    Some(format!(r#"<path d="{}"/>"#, d))
}
