use tiny_skia as sk;
use ttf_parser::GlyphId;
use typst::{
    doc::TextItem,
    font::Font,
    geom::{Abs, Axes, Paint, Size},
    image::Image,
};
use typst_ts_core::error::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::Path2d;

use crate::{
    svg::SvgPath2DBuilder,
    utils::{AbsExt, CanvasStateGuard, ToCssExt},
    CanvasRenderTask,
};

impl<'a> CanvasRenderTask<'a> {
    /// Render a text run into the self.canvas.
    pub(crate) fn render_text(&mut self, ts: sk::Transform, text: &TextItem) {
        let glyph_chars: String = if text.glyphs.is_empty() {
            "".to_string()
        } else {
            text.text[text.glyphs[0].range().start..text.glyphs[text.glyphs.len() - 1].range().end]
                .to_string()
        };

        let mut x = 0.0;
        for glyph in &text.glyphs {
            let id = GlyphId(glyph.id);
            let offset = x + glyph.x_offset.at(text.size).to_f32();
            let ts = ts.pre_translate(offset, 0.0);

            self.render_svg_glyph(ts, text, id)
                .or_else(|| self.render_bitmap_glyph(ts, text, id))
                .or_else(|| self.render_outline_glyph(ts, text, id));

            x += glyph.x_advance.at(text.size).to_f32();
        }

        self.append_text_content(ts, text, glyph_chars, x, text.size.to_f32(), false)
    }

    /// Render an SVG glyph into the self.canvas.
    // todo: verify correctness
    // todo: use HtmlSvgElement
    fn render_svg_glyph(&mut self, ts: sk::Transform, text: &TextItem, id: GlyphId) -> Option<()> {
        let data = text.font.ttf().glyph_svg_image(id)?;

        // If there's no viewbox defined, use the em square for our scale
        // transformation ...
        let upem = text.font.units_per_em() as f32;
        let (width, height) = (upem, upem);

        // // ... but if there's a viewbox or width, use that.
        // if root.has_attribute("viewBox") || root.has_attribute("width") {
        //     width = view_box.width() as f32;
        // }

        // // Same as for width.
        // if root.has_attribute("viewBox") || root.has_attribute("height") {
        //     height = view_box.height() as f32;
        // }

        let size = text.size.to_f32();
        let ts = ts.pre_scale(size / width, size / height);

        // todo: paint
        // &sk::PixmapPaint::default(),

        let image = Image::new_raw(
            data.into(),
            typst::image::ImageFormat::Vector(typst::image::VectorFormat::Svg),
            Axes::new(width as u32, height as u32),
            None,
        )
        .ok()?;

        self.render_image(
            ts,
            &image,
            Size::new(Abs::pt(width as f64), Abs::pt(height as f64)),
        );
        Some(())
    }

    /// Render a bitmap glyph into the self.canvas.
    // todo: verify correctness
    fn render_bitmap_glyph(
        &mut self,
        ts: sk::Transform,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let size = text.size.to_f32();
        let ppem = size * ts.sy;

        let raster = text.font.ttf().glyph_raster_image(id, ppem as u16)?;
        let image = Image::new_raw(
            raster.data.into(),
            raster.format.into(),
            Axes::new(raster.x as u32, raster.y as u32),
            None,
        )
        .ok()?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?
        let h = text.size;
        let w = (image.width() as f64 / image.height() as f64) * h;

        let dx = (raster.x as f32) / (image.width() as f32) * size;
        let dy = (raster.y as f32) / (image.height() as f32) * size;
        let ts = ts.pre_translate(dx, -size - dy);

        self.render_image(ts, &image, Size::new(w, h));
        Some(())
    }

    /// Render an outline glyph into the canvas. This is the "normal" case.
    fn render_outline_glyph(
        &mut self,
        ts: sk::Transform,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let state_guard = CanvasStateGuard::new(self.canvas);

        // Scale is in pixel per em, but curve data is in font design units, so
        // we have to divide by units per em.
        let ppem = {
            let pixel_per_unit = text.size.to_f32();
            let units_per_em = text.font.ttf().units_per_em() as f32;
            pixel_per_unit / units_per_em
        };

        // todo: error handling, reuse color
        self.set_transform(ts.pre_scale(ppem, -ppem));

        {
            self.canvas.begin_path();

            let Paint::Solid(color) = text.fill;
            self.canvas.set_fill_style(&color.to_css().into());

            let path = self.collect_err(
                extract_outline_glyph(&text.font, id)?
                    .map_err(map_err("CanvasRenderTask.BuildPath2d")),
            )?;
            self.canvas.fill_with_path_2d(&path);
        }
        drop(state_guard);

        Some(())
    }
}

#[comemo::memoize]
fn extract_outline_glyph(font: &Font, id: GlyphId) -> Option<Result<Path2d, JsValue>> {
    // todo: handling no such glyph
    let mut builder = SvgPath2DBuilder(String::new());
    font.ttf().outline_glyph(id, &mut builder)?;
    Some(builder.build())
}
