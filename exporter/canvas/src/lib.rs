//! Rendering into raster images.

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::io::Read;
use std::sync::Arc;

use image::imageops::FilterType;
use image::{GenericImageView, Rgba};
use tiny_skia as sk;
use ttf_parser::{GlyphId, OutlineBuilder};
use usvg::{FitTo, NodeExt};

use typst::doc::{Frame, FrameItem, GroupItem, Meta, TextItem};
use typst::geom::{self, Abs, Color, Geometry, Paint, PathItem, Shape, Size, Stroke};
use typst::image::{DecodedImage, Image};
use wasm_bindgen::Clamped;
use web_sys::ImageData;

pub struct CanvasRenderTask<'a> {
    canvas: &'a web_sys::CanvasRenderingContext2d,

    pixel_per_pt: f32,
    fill: Color,

    width: u32,
    height: u32,
}

impl<'a> CanvasRenderTask<'a> {
    pub fn new(
        canvas: &'a web_sys::CanvasRenderingContext2d,
        doc: &'a typst::doc::Document,
        page_off: usize,
        pixel_per_pt: f32,
        fill: Color,
    ) -> Self {
        let size = doc.pages[page_off].size();
        let pxw = (pixel_per_pt * (size.x.to_pt() as f32)).round().max(1.0) as u32;
        let pxh = (pixel_per_pt * (size.y.to_pt() as f32)).round().max(1.0) as u32;
        Self {
            canvas,
            pixel_per_pt,
            fill,
            width: pxw,
            height: pxh,
        }
    }

    /// Export a frame into a raster image.
    ///
    /// This renders the frame at the given number of pixels per point and returns
    /// the resulting `tiny-skia` pixel buffer.
    pub fn render(&self, frame: &Frame) {
        let fill = self.fill.to_rgba();
        let fill = format!("rgba({}, {}, {}, {})", fill.r, fill.g, fill.b, fill.a);
        self.canvas.set_fill_style(&fill.into());
        self.canvas
            .fill_rect(0., 0., self.width as f64, self.height as f64);

        let ts = sk::Transform::from_scale(self.pixel_per_pt, self.pixel_per_pt);
        self.render_frame(ts, None, frame);
    }

    /// Render a frame into the canvas.
    fn render_frame(&self, ts: sk::Transform, mask: Option<&sk::ClipMask>, frame: &Frame) {
        for (pos, item) in frame.items() {
            let x = pos.x.to_f32();
            let y = pos.y.to_f32();
            let ts = ts.pre_translate(x, y);

            match item {
                FrameItem::Group(group) => {
                    // 300KB
                    self.render_group(ts, mask, group);
                }
                FrameItem::Text(text) => {
                    // 2300KB
                    self.render_text(ts, mask, text);
                }
                FrameItem::Shape(shape, _) => {
                    // 300KB
                    self.render_shape(ts, mask, shape);
                }
                FrameItem::Image(image, size, _) => {
                    // 1300KB
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
    fn render_group(&self, ts: sk::Transform, mask: Option<&sk::ClipMask>, group: &GroupItem) {
        let ts = ts.pre_concat(group.transform.into());

        // let mut mask = mask;

        // todo: clip
        // let mut storage;
        // if group.clips {
        //     let size = group.frame.size();
        //     let w = size.x.to_f32();
        //     let h = size.y.to_f32();
        //     if let Some(path) = sk::Rect::from_xywh(0.0, 0.0, w, h)
        //         .map(sk::PathBuilder::from_rect)
        //         .and_then(|path| path.transform(ts))
        //     {
        //         let result = if let Some(mask) = mask {
        //             storage = mask.clone();
        //             storage.intersect_path(&path, sk::FillRule::default(), false)
        //         } else {
        //             let pxw = self.width;
        //             let pxh = self.height;
        //             storage = sk::ClipMask::new();
        //             storage.set_path(pxw, pxh, &path, sk::FillRule::default(), false)
        //         };

        //         // Clipping fails if clipping rect is empty. In that case we just
        //         // clip everything by returning.
        //         if result.is_none() {
        //             return;
        //         }

        //         mask = Some(&storage);
        //     }
        // }

        self.render_frame(ts, mask, &group.frame);
    }

    /// Render a text run into the self.canvas.
    fn render_text(&self, ts: sk::Transform, mask: Option<&sk::ClipMask>, text: &TextItem) {
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
    }

    /// Render an SVG glyph into the self.canvas.
    fn render_svg_glyph(
        &self,
        ts: sk::Transform,
        mask: Option<&sk::ClipMask>,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let mut data = text.font.ttf().glyph_svg_image(id)?;

        // Decompress SVGZ.
        let mut decoded = vec![];
        if data.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = flate2::read::GzDecoder::new(data);
            decoder.read_to_end(&mut decoded).ok()?;
            data = &decoded;
        }

        // Parse XML.
        let xml = std::str::from_utf8(data).ok()?;
        let document = roxmltree::Document::parse(xml).ok()?;
        let root = document.root_element();

        // Parse SVG.
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_xmltree(&document, &opts.to_ref()).ok()?;
        let view_box = tree.svg_node().view_box.rect;

        // If there's no viewbox defined, use the em square for our scale
        // transformation ...
        let upem = text.font.units_per_em() as f32;
        let (mut width, mut height) = (upem, upem);

        // ... but if there's a viewbox or width, use that.
        if root.has_attribute("viewBox") || root.has_attribute("width") {
            width = view_box.width() as f32;
        }

        // Same as for width.
        if root.has_attribute("viewBox") || root.has_attribute("height") {
            height = view_box.height() as f32;
        }

        let size = text.size.to_f32();
        let ts = ts.pre_scale(size / width, size / height);

        // Compute the space we need to draw our glyph.
        // See https://github.com/RazrFalcon/resvg/issues/602 for why
        // using the svg size is problematic here.
        let mut bbox = usvg::Rect::new_bbox();
        for node in tree.root().descendants() {
            if let Some(rect) = node.calculate_bbox().and_then(|b| b.to_rect()) {
                bbox = bbox.expand(rect);
            }
        }

        let canvas_rect = usvg::ScreenRect::new(0, 0, self.width, self.height)?;

        // Compute the bbox after the transform is applied.
        // We add a nice 5px border along the bounding box to
        // be on the safe size. We also compute the intersection
        // with the self.canvas rectangle
        let svg_ts = usvg::Transform::new(
            ts.sx.into(),
            ts.kx.into(),
            ts.ky.into(),
            ts.sy.into(),
            ts.tx.into(),
            ts.ty.into(),
        );
        let bbox = bbox.transform(&svg_ts)?.to_screen_rect();
        let bbox = usvg::ScreenRect::new(
            bbox.left() - 5,
            bbox.y() - 5,
            bbox.width() + 10,
            bbox.height() + 10,
        )?
        .fit_to_rect(canvas_rect);
        // let (prealloc, size) = self.render_to_image_internal(session, options)?;

        // Ok(ImageData::new_with_u8_clamped_array_and_sh(
        //     Clamped(prealloc.as_slice()),
        //     size.width,
        //     size.height,
        // )?)

        let mut pixmap = sk::Pixmap::new(bbox.width(), bbox.height())?;

        // We offset our transform so that the pixmap starts at the edge of the bbox.
        let ts = ts.post_translate(-bbox.left() as f32, -bbox.top() as f32);
        resvg::render(&tree, FitTo::Original, ts, pixmap.as_mut())?;

        // Draws a `Pixmap` on top of the current `Pixmap`.
        //
        // We basically filling a rectangle with a `pixmap` pattern.
        // pub fn draw_pixmap(
        //     &mut self,
        //     x: i32,
        //     y: i32,
        //     pixmap: PixmapRef,
        //     paint: &PixmapPaint,
        //     transform: Transform,
        //     clip_mask: Option<&ClipMask>,
        // ) -> Option<()> {
        // ctx.drawImage(YOUR_MASK, 0, 0);
        // ctx.globalCompositeOperation = 'source-in';
        // ctx.drawImage(YOUR_IMAGE, 0 , 0);

        // self.canvas.put_image_data(mask, dx, dy)

        // todo: paint
        // &sk::PixmapPaint::default(),
        // todo: transform
        // sk::Transform::identity(),
        // todo: mask
        // mask,

        // todo: error handling
        let web_img = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(pixmap.data()),
            bbox.width(),
            bbox.height(),
        )
        .unwrap();

        // todo: error handling
        self.canvas
            .put_image_data(&web_img, bbox.left() as f64, bbox.top() as f64)
            .ok()
    }

    /// Render a bitmap glyph into the self.canvas.
    fn render_bitmap_glyph(
        &self,
        ts: sk::Transform,
        mask: Option<&sk::ClipMask>,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let size = text.size.to_f32();
        let ppem = size * ts.sy;
        let raster = text.font.ttf().glyph_raster_image(id, ppem as u16)?;
        let image = Image::new(raster.data.into(), raster.format.into(), None).ok()?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?
        let h = text.size;
        let w = (image.width() as f64 / image.height() as f64) * h;
        let dx = (raster.x as f32) / (image.width() as f32) * size;
        let dy = (raster.y as f32) / (image.height() as f32) * size;
        let ts = ts.pre_translate(dx, -size - dy);
        self.render_image(ts, mask, &image, Size::new(w, h))
    }

    /// Render an outline glyph into the canvas. This is the "normal" case.
    fn render_outline_glyph(
        &self,
        ts: sk::Transform,
        mask: Option<&sk::ClipMask>,
        text: &TextItem,
        id: GlyphId,
    ) -> Option<()> {
        let ppem = text.size.to_f32() * ts.sy;

        // Render a glyph directly as a path. This only happens when the fast glyph
        // rasterization can't be used due to very large text size or weird
        // scale/skewing transforms.
        if ppem > 100.0 || ts.kx != 0.0 || ts.ky != 0.0 || ts.sx != ts.sy {
            panic!("big outline glyph");
            // let path = {
            //     let mut builder = WrappedPathBuilder(sk::PathBuilder::new());
            //     text.font.ttf().outline_glyph(id, &mut builder)?;
            //     builder.0.finish()?
            // };

            // let paint = (&text.fill).into();
            // let rule = sk::FillRule::default();

            // // Flip vertically because font design coordinate
            // // system is Y-up.
            // let scale = text.size.to_f32() / text.font.units_per_em() as f32;
            // let ts = ts.pre_scale(scale, -scale);
            // self.canvas.fill_path(&path, &paint, rule, ts, mask)?;
            // return Some(());
        }

        // Rasterize the glyph with `pixglyph`.
        // Try to retrieve a prepared glyph or prepare it from scratch if it
        // doesn't exist, yet.
        let glyph = pixglyph::Glyph::load(text.font.ttf(), id)?;
        let bitmap = glyph.rasterize(ts.tx, ts.ty, ppem);

        // todo: mask
        // If we have a clip mask we first render to a pixmap that we then blend
        // with our canvas
        // if mask.is_some() {
        // } else {
        //     let cw = self.width as i32;
        //     let ch = self.height as i32;
        //     let mw = bitmap.width as i32;
        //     let mh = bitmap.height as i32;

        //     // Determine the pixel bounding box that we actually need to draw.
        //     let left = bitmap.left;
        //     let right = left + mw;
        //     let top = bitmap.top;
        //     let bottom = top + mh;

        //     // Premultiply the text color.
        //     let Paint::Solid(color) = text.fill;
        //     let c = color.to_rgba();
        //     let color = sk::ColorU8::from_rgba(c.r, c.g, c.b, 255)
        //         .premultiply()
        //         .get();

        //     // Blend the glyph bitmap with the existing pixels on the canvas.
        //     let pixels = bytemuck::cast_slice_mut::<u8, u32>(canvas.data_mut());
        //     for x in left.clamp(0, cw)..right.clamp(0, cw) {
        //         for y in top.clamp(0, ch)..bottom.clamp(0, ch) {
        //             let ai = ((y - top) * mw + (x - left)) as usize;
        //             let cov = bitmap.coverage[ai];
        //             if cov == 0 {
        //                 continue;
        //             }

        //             let pi = (y * cw + x) as usize;
        //             if cov == 255 {
        //                 pixels[pi] = color;
        //                 continue;
        //             }

        //             let applied = alpha_mul(color, cov as u32);
        //             pixels[pi] = blend_src_over(applied, pixels[pi]);
        //         }
        //     }

        //     Some(())
        // }

        let mw = bitmap.width;
        let mh = bitmap.height;

        let Paint::Solid(color) = text.fill;
        let c = color.to_rgba();

        // Pad the pixmap with 1 pixel in each dimension so that we do
        // not get any problem with floating point errors along their border
        let mut pixmap = sk::Pixmap::new(mw + 2, mh + 2)?;
        for x in 0..mw {
            for y in 0..mh {
                let alpha = bitmap.coverage[(y * mw + x) as usize];
                let color = sk::ColorU8::from_rgba(c.r, c.g, c.b, alpha).premultiply();
                pixmap.pixels_mut()[((y + 1) * (mw + 2) + (x + 1)) as usize] = color;
            }
        }

        let left = bitmap.left;
        let top = bitmap.top;

        // todo: paint
        // &sk::PixmapPaint::default(),
        // todo: transform
        // sk::Transform::identity(),
        // todo: mask
        // mask,

        // todo: error handling
        let web_img =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(pixmap.data()), mw + 2, mh + 2)
                .unwrap();

        // todo handle error
        self.canvas
            .put_image_data(&web_img, (left - 1) as f64, (top - 1) as f64)
            .ok()
    }

    /// Render a geometrical shape into the canvas.
    fn render_shape(
        &self,
        ts: sk::Transform,
        mask: Option<&sk::ClipMask>,
        shape: &Shape,
    ) -> Option<()> {
        // todo: shape
        // let path = match shape.geometry {
        //     Geometry::Line(target) => {
        //         let mut builder = sk::PathBuilder::new();
        //         builder.line_to(target.x.to_f32(), target.y.to_f32());
        //         builder.finish()?
        //     }
        //     Geometry::Rect(size) => {
        //         let w = size.x.to_f32();
        //         let h = size.y.to_f32();
        //         let rect = sk::Rect::from_xywh(0.0, 0.0, w, h)?;
        //         sk::PathBuilder::from_rect(rect)
        //     }
        //     Geometry::Path(ref path) => self.convert_path(path)?,
        // };

        // if let Some(fill) = &shape.fill {
        //     let mut paint: sk::Paint = fill.into();
        //     if matches!(shape.geometry, Geometry::Rect(_)) {
        //         paint.anti_alias = false;
        //     }

        //     let rule = sk::FillRule::default();
        //     canvas.fill_path(&path, &paint, rule, ts, mask);
        // }

        // if let Some(Stroke {
        //     paint,
        //     thickness,
        //     line_cap,
        //     line_join,
        //     dash_pattern,
        //     miter_limit,
        // }) = &shape.stroke
        // {
        //     let dash = dash_pattern.as_ref().and_then(|pattern| {
        //         // tiny-skia only allows dash patterns with an even number of elements,
        //         // while pdf allows any number.
        //         let len = if pattern.array.len() % 2 == 1 {
        //             pattern.array.len() * 2
        //         } else {
        //             pattern.array.len()
        //         };
        //         let dash_array = pattern
        //             .array
        //             .iter()
        //             .map(|l| l.to_f32())
        //             .cycle()
        //             .take(len)
        //             .collect();

        //         sk::StrokeDash::new(dash_array, pattern.phase.to_f32())
        //     });
        //     let paint = paint.into();
        //     let stroke = sk::Stroke {
        //         width: thickness.to_f32(),
        //         line_cap: line_cap.into(),
        //         line_join: line_join.into(),
        //         dash,
        //         miter_limit: miter_limit.0 as f32,
        //     };
        //     self.canvas.stroke_path(&path, &paint, &stroke, ts, mask);
        // }

        Some(())
    }

    /// Convert a Typst path into a tiny-skia path.
    fn convert_path(&self, path: &geom::Path) -> Option<sk::Path> {
        let mut builder = sk::PathBuilder::new();
        for elem in &path.0 {
            match elem {
                PathItem::MoveTo(p) => {
                    builder.move_to(p.x.to_f32(), p.y.to_f32());
                }
                PathItem::LineTo(p) => {
                    builder.line_to(p.x.to_f32(), p.y.to_f32());
                }
                PathItem::CubicTo(p1, p2, p3) => {
                    builder.cubic_to(
                        p1.x.to_f32(),
                        p1.y.to_f32(),
                        p2.x.to_f32(),
                        p2.y.to_f32(),
                        p3.x.to_f32(),
                        p3.y.to_f32(),
                    );
                }
                PathItem::ClosePath => {
                    builder.close();
                }
            };
        }
        builder.finish()
    }

    /// Render a raster or SVG image into the canvas.
    fn render_image(
        &self,
        ts: sk::Transform,
        mask: Option<&sk::ClipMask>,
        image: &Image,
        size: Size,
    ) -> Option<()> {
        panic!("render_image");
        //     let view_width = size.x.to_f32();
        //     let view_height = size.y.to_f32();

        //     let aspect = (image.width() as f32) / (image.height() as f32);
        //     let scale = ts.sx.max(ts.sy);
        //     let w = (scale * view_width.max(aspect * view_height)).ceil() as u32;
        //     let h = ((w as f32) / aspect).ceil() as u32;

        //     let pixmap = scaled_texture(image, w, h)?;
        //     let scale_x = view_width / pixmap.width() as f32;
        //     let scale_y = view_height / pixmap.height() as f32;

        //     let paint = sk::Paint {
        //         shader: sk::Pattern::new(
        //             (*pixmap).as_ref(),
        //             sk::SpreadMode::Pad,
        //             sk::FilterQuality::Nearest,
        //             1.0,
        //             sk::Transform::from_scale(scale_x, scale_y),
        //         ),
        //         ..Default::default()
        //     };

        //     let rect = sk::Rect::from_xywh(0.0, 0.0, view_width, view_height)?;
        //     self.canvas.fill_rect(rect, &paint, ts, mask);

        //     Some(())
    }
}

/// Prepare a texture for an image at a scaled size.
fn scaled_texture(image: &Image, w: u32, h: u32) -> Option<Arc<sk::Pixmap>> {
    let mut pixmap = sk::Pixmap::new(w, h)?;
    match image.decoded() {
        DecodedImage::Raster(dynamic, _) => {
            let downscale = w < image.width();
            let filter = if downscale {
                FilterType::Lanczos3
            } else {
                FilterType::CatmullRom
            };
            let buf = dynamic.resize(w, h, filter);
            for ((_, _, src), dest) in buf.pixels().zip(pixmap.pixels_mut()) {
                let Rgba([r, g, b, a]) = src;
                *dest = sk::ColorU8::from_rgba(r, g, b, a).premultiply();
            }
        }
        DecodedImage::Svg(tree) => {
            resvg::render(
                tree,
                FitTo::Size(w, h),
                sk::Transform::identity(),
                pixmap.as_mut(),
            )?;
        }
    }
    Some(Arc::new(pixmap))
}

/// Allows to build tiny-skia paths from glyph outlines.
struct WrappedPathBuilder(sk::PathBuilder);

impl OutlineBuilder for WrappedPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}

/// Additional methods for [`Length`].
trait AbsExt {
    /// Convert to a number of points as f32.
    fn to_f32(self) -> f32;
}

impl AbsExt for Abs {
    fn to_f32(self) -> f32 {
        self.to_pt() as f32
    }
}

// Alpha multiplication and blending are ported from:
// https://skia.googlesource.com/skia/+/refs/heads/main/include/core/SkColorPriv.h

/// Blends two premulitplied, packed 32-bit RGBA colors. Alpha channel must be
/// in the 8 high bits.
fn blend_src_over(src: u32, dst: u32) -> u32 {
    src + alpha_mul(dst, 256 - (src >> 24))
}

/// Alpha multiply a color.
fn alpha_mul(color: u32, scale: u32) -> u32 {
    let mask = 0xff00ff;
    let rb = ((color & mask) * scale) >> 8;
    let ag = ((color >> 8) & mask) * scale;
    (rb & mask) | (ag & !mask)
}
