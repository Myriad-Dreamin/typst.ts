use std::io::Read;

use once_cell::sync::OnceCell;
use tiny_skia as sk;
use ttf_parser::GlyphId;
use typst::{
    doc::TextItem,
    font::Font,
    geom::{Abs, Axes, Paint, Size},
    image::Image,
};
use typst_ts_core::{error::prelude::*, font::GlyphProvider};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Path2d};

use crate::{
    utils::{console_log, AbsExt, CanvasStateGuard, ToCssExt},
    CanvasRenderTask, RenderFeature,
};

static WARN_HARMFUL_FONT: OnceCell<()> = OnceCell::new();
static WARN_VIEW_BOX: OnceCell<()> = OnceCell::new();

impl<'a, Feat: RenderFeature> CanvasRenderTask<'a, Feat> {
    /// Render a text run into the self.canvas.
    pub(crate) async fn render_text(&mut self, ts: sk::Transform, text: &TextItem) {
        let _r = self.perf_event("render_text");
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

            let _ = self.render_svg_glyph(ts, text, id).await.is_some()
                || self.render_bitmap_glyph(ts, text, id).await.is_some()
                || self.render_outline_glyph(ts, text, id).is_some();

            x += glyph.x_advance.at(text.size).to_f32();
        }

        let _r = self.perf_event("append_text_content");
        self.append_text_content(ts, text, glyph_chars, x, text.size.to_f32(), false)
    }

    /// Render an SVG glyph into the self.canvas.
    /// More information: https://learn.microsoft.com/zh-cn/typography/opentype/spec/svg
    async fn render_svg_glyph(
        &mut self,
        ts: sk::Transform,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let _r = self.perf_event("render_svg_glyph");
        let font = &text.font;
        let glyph_image = extract_svg_glyph(self.glyph_provider.clone(), font, id)?;

        // position our image
        let ascender = font.metrics().ascender.at(text.size).to_f32();

        let size = text.size.to_f32();
        // make ascender back
        let ts = ts.pre_translate(0., -ascender);
        // pre scale to correct size
        let ts = ts.pre_scale(
            size / glyph_image.width() as f32,
            size / glyph_image.height() as f32,
        );

        self.render_image(
            ts,
            &glyph_image,
            Size::new(
                Abs::pt(glyph_image.width() as f64),
                Abs::pt(glyph_image.height() as f64),
            ),
        )
        .await;
        Some(())
    }

    /// Render a bitmap glyph into the self.canvas.
    async fn render_bitmap_glyph(
        &mut self,
        ts: sk::Transform,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let _r = self.perf_event("render_bitmap_glyph");
        let size = text.size.to_f32();
        let ppem = (size * ts.sy) as u16;

        let (glyph_image, raster_x, raster_y) =
            extract_bitmap_glyph(self.glyph_provider.clone(), &text.font, id, ppem)?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?
        let h = text.size;
        let w = (glyph_image.width() as f64 / glyph_image.height() as f64) * h;

        let dx = (raster_x as f32) / (glyph_image.width() as f32) * size;
        let dy = (raster_y as f32) / (glyph_image.height() as f32) * size;
        let ts = ts.pre_translate(dx, -size - dy);

        self.render_image(ts, &glyph_image, Size::new(w, h)).await;
        Some(())
    }

    /// Render an outline glyph into the canvas. This is the "normal" case.
    fn render_outline_glyph(
        &mut self,
        ts: sk::Transform,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let _r = self.perf_event("render_outline_glyph");
        let state_guard = CanvasStateGuard::new(self.canvas);

        // Scale is in pixel per em, but curve data is in font design units, so
        // we have to divide by units per em.
        let ppem = {
            let pixel_per_unit = text.size.to_f32();
            let units_per_em = text.font.units_per_em() as f32;
            pixel_per_unit / units_per_em
        };

        if cfg!(feature = "debug_glyph_render") {
            console_log!("render_outline_glyph: {:?} {:?}", text.font.info(), ppem);
        }

        // todo: error handling, reuse color
        self.set_transform(ts.pre_scale(ppem, -ppem));
        {
            self.canvas.begin_path();

            let Paint::Solid(color) = text.fill;
            self.canvas.set_fill_style(&color.to_css().into());

            let path = self.collect_err(
                extract_outline_glyph(self.glyph_provider.clone(), &text.font, id)?
                    .map_err(map_err("CanvasRenderTask.BuildPath2d")),
            )?;
            self.canvas.fill_with_path_2d(&path);
        }
        drop(state_guard);

        Some(())
    }
}

#[comemo::memoize]
fn extract_svg_glyph(g: GlyphProvider, font: &Font, id: GlyphId) -> Option<Image> {
    let data = g.svg_glyph(font, id)?;
    let mut data = data.as_ref();

    let font_info = font.info();
    let font_family = &font_info.family;
    let font_metrics = font.metrics();

    if cfg!(feature = "debug_glyph_render") {
        console_log!(
            "render_svg_glyph: {:?} {:?}",
            font.info().family,
            font_metrics
        );
    }

    // Decompress SVGZ.
    let mut decoded = vec![];
    // The first three bytes of the gzip-encoded document header must be 0x1F, 0x8B, 0x08.
    if data.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = flate2::read::GzDecoder::new(data);
        decoder.read_to_end(&mut decoded).ok()?;
        data = &decoded;
    }

    // todo: When a font engine renders glyph 14, the result shall be the same as rendering the following SVG document
    //   <svg> <defs> <use #glyph{id}> </svg>

    let upem = Abs::raw(font.units_per_em());
    let (width, height) = (upem.to_f32(), upem.to_f32());
    let origin_ascender = font_metrics.ascender.at(upem).to_f32();

    let doc_string = String::from_utf8(data.to_owned()).unwrap();

    let dom_parser = web_sys::DomParser::new().unwrap();
    let doc = dom_parser
        .parse_from_string(&doc_string, web_sys::SupportedType::ImageSvgXml)
        .unwrap();

    // todo: verify SVG capability requirements and restrictions
    // Use of relative units em, ex
    // Use of SVG data within <image> elements
    // Use of color profiles (the <icccolor> data type, the <color-profile> element, the color-profile property, or the CSS @color-profile rule)
    // Use of the contentStyleType attribute
    // Use of CSS2 system color keywords
    let contains_elem = |tag| doc.get_elements_by_tag_name(tag).length() > 0;
    if doc
        .query_selector("text,foreignObject,switch,script,a,view")
        .unwrap()
        .is_some()
        || contains_elem("xsl:processing-instruction")
    {
        WARN_HARMFUL_FONT.get_or_init(|| {
            web_sys::console::warn_2(
                &"warning(typst.ts,canvas): this font may be harmful".into(),
                &font_family.into(),
            );
        });

        // fallback
        return None;
    }

    // get svg
    let root = doc.first_child().unwrap();
    let svg: &Element = root.dyn_ref()?;

    // update view box
    let view_box = svg
        .get_attribute("viewBox")
        .map(|s| {
            WARN_VIEW_BOX.get_or_init(|| {
                console_log!(
                    "render_svg_glyph with viewBox, This should be helpful if you can help us verify the result: {:?} {:?}",
                    font.info().family,
                    doc_string
                );
            });
            s
        })
        .unwrap_or_else(|| format!("0 {} {} {}", -origin_ascender, width, height));
    svg.set_attribute("viewBox", &view_box).unwrap();

    let doc_string = svg.outer_html();

    if cfg!(feature = "debug_glyph_render") {
        console_log!(
            "render_svg_glyph prepared document: {:?} {:?}",
            font.info().family,
            doc_string
        );
    }

    let image = Image::new_raw(
        doc_string.as_bytes().to_vec().into(),
        typst::image::ImageFormat::Vector(typst::image::VectorFormat::Svg),
        Axes::new(width as u32, height as u32),
        None,
    )
    .ok()?;

    Some(image)
}

#[comemo::memoize]
fn extract_bitmap_glyph(
    g: GlyphProvider,
    font: &Font,
    id: GlyphId,
    ppem: u16,
) -> Option<(Image, i16, i16)> {
    // if cfg!(feature = "debug_glyph_render") {
    //     console_log!(
    //         "render_bitmap_glyph: {:?} {:?}x{:?}",
    //         font.info().family,
    //         raster.width,
    //         raster.height
    //     );
    // }

    g.bitmap_glyph(font, id, ppem)
}

#[comemo::memoize]
fn extract_outline_glyph(
    g: GlyphProvider,
    font: &Font,
    id: GlyphId,
) -> Option<Result<Path2d, JsValue>> {
    Some(Path2d::new_with_path_string(&g.outline_glyph(font, id)?))
}
