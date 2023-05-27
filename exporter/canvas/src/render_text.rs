use tiny_skia as sk;
use ttf_parser::GlyphId;
use typst::{doc::TextItem, font::Font, geom::Paint};
use web_sys::Path2d;

use crate::{
    svg::SvgPath2DBuilder,
    utils::{AbsExt, CanvasStateGuard, ToCssExt},
    CanvasRenderTask,
};

impl<'a> CanvasRenderTask<'a> {
    /// Render a text run into the self.canvas.
    pub(crate) fn render_text(
        &mut self,
        ts: sk::Transform,
        mask: Option<&sk::Mask>,
        text: &TextItem,
    ) {
        let info = text.font.info();

        let glyph_chars: String = if text.glyphs.is_empty() {
            "".to_string()
        } else {
            text.text[text.glyphs[0].range().start..text.glyphs[text.glyphs.len() - 1].range().end]
                .to_string()
        };
        let ppem = text.size.to_f32() * ts.sy;

        // console_log!("render text {:?}", glyph_chars);

        let mut x = 0.0;
        for glyph in &text.glyphs {
            let id = GlyphId(glyph.id);
            let offset = x + glyph.x_offset.at(text.size).to_f32();
            let ts = ts.pre_translate(offset, 0.0);

            self.render_svg_glyph(ts, mask, text, id)
                .or_else(|| self.render_bitmap_glyph(ts, mask, text, id))
                .or_else(|| self.render_outline_glyph(ts, mask, text, id));

            x += glyph.x_advance.at(text.size).to_f32();
        }

        self.append_text_content(ts, text, glyph_chars, x, text.size.to_f32(), false)
    }

    /// Render an SVG glyph into the self.canvas.
    fn render_svg_glyph(
        &mut self,
        ts: sk::Transform,
        mask: Option<&sk::Mask>,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let data = text.font.ttf().glyph_svg_image(id)?;
        panic!("rendering svg glyph {}", id.0);

        // // Decompress SVGZ.
        // let mut decoded = vec![];
        // if data.starts_with(&[0x1f, 0x8b]) {
        //     let mut decoder = flate2::read::GzDecoder::new(data);
        //     decoder.read_to_end(&mut decoded).ok()?;
        //     data = &decoded;
        // }

        // // Parse XML.
        // let xml = std::str::from_utf8(data).ok()?;
        // let document = roxmltree::Document::parse(xml).ok()?;
        // let root = document.root_element();

        // // Parse SVG.
        // let opts = usvg::Options::default();
        // let tree = usvg::Tree::from_xmltree(&document, &opts.to_ref()).ok()?;
        // let view_box = tree.svg_node().view_box.rect;

        // // If there's no viewbox defined, use the em square for our scale
        // // transformation ...
        // let upem = text.font.units_per_em() as f32;
        // let (mut width, mut height) = (upem, upem);

        // // ... but if there's a viewbox or width, use that.
        // if root.has_attribute("viewBox") || root.has_attribute("width") {
        //     width = view_box.width() as f32;
        // }

        // // Same as for width.
        // if root.has_attribute("viewBox") || root.has_attribute("height") {
        //     height = view_box.height() as f32;
        // }

        // let size = text.size.to_f32();
        // let ts = ts.pre_scale(size / width, size / height);

        // // Compute the space we need to draw our glyph.
        // // See https://github.com/RazrFalcon/resvg/issues/602 for why
        // // using the svg size is problematic here.
        // let mut bbox = usvg::Rect::new_bbox();
        // for node in tree.root().descendants() {
        //     if let Some(rect) = node.calculate_bbox().and_then(|b| b.to_rect()) {
        //         bbox = bbox.expand(rect);
        //     }
        // }

        // let canvas_rect = usvg::ScreenRect::new(0, 0, self.width, self.height)?;

        // // Compute the bbox after the transform is applied.
        // // We add a nice 5px border along the bounding box to
        // // be on the safe size. We also compute the intersection
        // // with the self.canvas rectangle
        // let svg_ts = usvg::Transform::new(
        //     ts.sx.into(),
        //     ts.kx.into(),
        //     ts.ky.into(),
        //     ts.sy.into(),
        //     ts.tx.into(),
        //     ts.ty.into(),
        // );
        // let bbox = bbox.transform(&svg_ts)?.to_screen_rect();
        // let bbox = usvg::ScreenRect::new(
        //     bbox.left() - 5,
        //     bbox.y() - 5,
        //     bbox.width() + 10,
        //     bbox.height() + 10,
        // )?
        // .fit_to_rect(canvas_rect);
        // // let (prealloc, size) = self.render_to_image_internal(session, options)?;

        // // Ok(ImageData::new_with_u8_clamped_array_and_sh(
        // //     Clamped(prealloc.as_slice()),
        // //     size.width,
        // //     size.height,
        // // )?)

        // let mut pixmap = sk::Pixmap::new(bbox.width(), bbox.height())?;

        // // We offset our transform so that the pixmap starts at the edge of the bbox.
        // let ts = ts.post_translate(-bbox.left() as f32, -bbox.top() as f32);
        // resvg::render(&tree, FitTo::Original, ts, pixmap.as_mut())?;

        // // Draws a `Pixmap` on top of the current `Pixmap`.
        // //
        // // We basically filling a rectangle with a `pixmap` pattern.
        // // pub fn draw_pixmap(
        // //     &mut self,
        // //     x: i32,
        // //     y: i32,
        // //     pixmap: PixmapRef,
        // //     paint: &PixmapPaint,
        // //     transform: Transform,
        // //     clip_mask: Option<&ClipMask>,
        // // ) -> Option<()> {
        // // ctx.drawImage(YOUR_MASK, 0, 0);
        // // ctx.globalCompositeOperation = 'source-in';
        // // ctx.drawImage(YOUR_IMAGE, 0 , 0);

        // // self.canvas.put_image_data(mask, dx, dy)

        // // todo: paint
        // // &sk::PixmapPaint::default(),
        // // todo: transform
        // // sk::Transform::identity(),
        // // todo: mask
        // // mask,

        // // todo: error handling
        // let web_img = ImageData::new_with_u8_clamped_array_and_sh(
        //     Clamped(pixmap.data()),
        //     bbox.width(),
        //     bbox.height(),
        // )
        // .unwrap();

        // // todo: error handling
        // self.canvas
        //     .put_image_data(&web_img, bbox.left() as f64, bbox.top() as f64)
        //     .ok()
    }

    /// Render a bitmap glyph into the self.canvas.
    fn render_bitmap_glyph(
        &mut self,
        ts: sk::Transform,
        mask: Option<&sk::Mask>,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let size = text.size.to_f32();
        let ppem = size * ts.sy;
        let raster = text.font.ttf().glyph_raster_image(id, ppem as u16)?;
        panic!("rendering bitmap glyph {}", id.0);
        // let image = Image::new(raster.data.into(), raster.format.into(), None).ok()?;
        // console_log!("rendering bitmap glyph {}", id.0);

        // // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // // and maybe also for Noto Color Emoji. And: Is the size calculation
        // // correct?
        // let h = text.size;
        // let w = (image.width() as f64 / image.height() as f64) * h;
        // let dx = (raster.x as f32) / (image.width() as f32) * size;
        // let dy = (raster.y as f32) / (image.height() as f32) * size;
        // let ts = ts.pre_translate(dx, -size - dy);
        // self.render_image(ts, mask, &image, Size::new(w, h))
    }

    /// Render an outline glyph into the canvas. This is the "normal" case.
    fn render_outline_glyph(
        &mut self,
        ts: sk::Transform,
        mask: Option<&sk::Mask>,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        // todo: big path, mask

        let ppem = text.size.to_f32() * ts.sy;

        if ts.kx != 0.0 || ts.ky != 0.0 || ts.sx != ts.sy {
            // panic!("skia does not support non-uniform scaling or skewing");
            return Some(()); // todo: don't submit
        }

        let state_guard = CanvasStateGuard::new(self.canvas);

        let face = text.font.ttf();

        // Scale is in pixel per em, but curve data is in font design units, so
        // we have to divide by units per em.
        let text_scale = ppem / face.units_per_em() as f32;

        // todo: error handling, reuse color, transform, reuse path2d objects
        self.canvas.reset_transform().unwrap();
        self.canvas.translate(ts.tx as f64, ts.ty as f64).unwrap();
        self.sync_transform(sk::Transform::from_scale(text_scale, -text_scale));
        // self.sync_transform(ts.pre_scale(text_scale, -text_scale));
        {
            self.canvas.begin_path();

            let Paint::Solid(color) = text.fill;
            self.canvas.set_fill_style(&color.to_css().into());
            self.canvas
                .fill_with_path_2d(&extract_outline_glyph(&text.font, id)?);
        }
        drop(state_guard);

        // todo: paint
        // &sk::PixmapPaint::default(),
        // todo: transform
        // sk::Transform::identity(),
        // todo: mask
        // mask,

        Some(())
    }
}

#[comemo::memoize]
fn extract_outline_glyph(font: &Font, id: GlyphId) -> Option<Path2d> {
    // todo: handling no such glyph
    let mut builder = SvgPath2DBuilder(String::new());
    font.ttf().outline_glyph(id, &mut builder)?;
    Some(Path2d::new_with_path_string(&builder.0).unwrap())
}
