//! Rendering into web_sys::CanvasRenderingContext2d.

#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt::Write;

// use js_sys::Reflect;
use tiny_skia as sk;
use ttf_parser::{GlyphId, OutlineBuilder};

use typst::doc::{Frame, FrameItem, GroupItem, Meta, TextItem};
use typst::font::{FontFlags, FontInfo, FontVariant};
use typst::geom::{self, Abs, Axis, Color, Dir, Geometry, Paint, PathItem, Shape, Size, Stroke};
use typst::image::{Image, ImageFormat, RasterFormat, VectorFormat};
use typst_ts_core::{content, TextContent};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, Path2d};

mod utils;

pub struct CanvasRenderTask<'a> {
    canvas: &'a CanvasRenderingContext2d,

    pixel_per_pt: f32,
    fill: Color,

    width: u32,
    height: u32,
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
    ) -> Self {
        if pixel_per_pt <= 0. {
            panic!("pixel_per_pt must be greater than 0");
        }
        let x = (js_sys::Math::random() * (0x7fffffff as f64)).ceil() as u64;
        let y = (js_sys::Math::random() * (0x7fffffff as f64)).ceil() as u64;
        let session_id = format!("{:x}", (x << 32) | y);
        canvas
            .canvas()
            .unwrap()
            .set_attribute("data-typst-session", &session_id)
            .unwrap();

        let size = doc.pages[page_off].size();
        let pxw = (pixel_per_pt * (size.x.to_pt() as f32)).round().max(1.0) as u32;
        let pxh = (pixel_per_pt * (size.y.to_pt() as f32)).round().max(1.0) as u32;
        Self {
            canvas,
            pixel_per_pt,
            fill,
            width: pxw,
            height: pxh,
            raw_height: pxh as f32 / pixel_per_pt,
            session_id,

            content: TextContent::default(),
            font_map: HashMap::default(),
        }
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

    /// Export a frame into a raster image.
    ///
    /// This renders the frame at the given number of pixels per point and returns
    /// the resulting `tiny-skia` pixel buffer.
    pub fn render(&mut self, frame: &Frame) {
        let fill = self.fill.to_rgba();
        let fill = format!("rgba({}, {}, {}, {})", fill.r, fill.g, fill.b, fill.a);
        self.canvas.set_fill_style(&fill.into());
        self.canvas
            .fill_rect(0., 0., self.width as f64, self.height as f64);

        let ts = sk::Transform::from_scale(self.pixel_per_pt, self.pixel_per_pt);
        self.render_frame(ts, None, frame);
    }

    /// Render a frame into the canvas.
    fn render_frame(&mut self, ts: sk::Transform, mask: Option<&sk::Mask>, frame: &Frame) {
        // let mut contains_text = false;
        let mut text_state = None;

        for (pos, item) in frame.items() {
            let x = pos.x.to_f32();
            let y = pos.y.to_f32();
            let ts = ts.pre_translate(x, y);

            match item {
                FrameItem::Group(group) => {
                    self.render_group(ts, mask, group);
                }
                FrameItem::Text(text) => {
                    let dir = text.lang.dir();

                    let mut has_eol = false;
                    if let Some((prev_dir, tx, ty, last_diff)) = text_state {
                        if prev_dir != dir {
                            text_state = Some((dir, ts.tx, ts.ty, None));
                            has_eol = true;
                        } else {
                            match dir.axis() {
                                Axis::X => {
                                    if ts.ty != ty {
                                        let diff = ts.ty - ty;
                                        text_state = Some((dir, ts.tx, ts.ty, Some(diff)));
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
                                        text_state = Some((dir, ts.tx, ts.ty, Some(diff)));
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
                        text_state = Some((dir, ts.tx, ts.ty, None));
                    }

                    if has_eol {
                        let font_name = self.write_font(&text.font);

                        let tts = ts.post_scale(1. / self.pixel_per_pt, 1. / self.pixel_per_pt);
                        self.content.items.push(content::TextItem {
                            str: "".to_string(),
                            // todo: real direction of the text
                            dir: match dir {
                                Dir::LTR => "ltr".to_string(),
                                Dir::RTL => "rtl".to_string(),
                                Dir::TTB => "ttb".to_string(),
                                Dir::BTT => "btt".to_string(),
                            },
                            width: 0.,
                            // todo: real height
                            height: 0.,
                            // todo: dirty pdf
                            transform: [
                                text.size.to_f32(),
                                tts.ky,
                                tts.kx,
                                text.size.to_f32(),
                                tts.tx,
                                self.raw_height - tts.ty,
                            ],
                            font_name,
                            has_eol,
                        });
                    }

                    self.render_text(ts, mask, text);
                    // contains_text = true;
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
        // if contains_text {
        // }
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
                    let pxw = self.width;
                    let pxh = self.height;
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

    fn write_font(&mut self, font: &typst::font::Font) -> u32 {
        if let Some(&font) = self.font_map.get(font.info()) {
            return font;
        }

        if self.content.styles.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let font_ref = self.content.styles.len() as u32;
        self.font_map.insert(font.info().clone(), font_ref);
        self.content.styles.push(content::TextStyle {
            font_family: font.info().family.clone(),
            ascent: font.metrics().ascender.get() as f32,
            descent: font.metrics().descender.get() as f32,
            vertical: false, // todo: check vertical
        });
        font_ref
    }

    /// Render a text run into the self.canvas.
    fn render_text(&mut self, ts: sk::Transform, mask: Option<&sk::Mask>, text: &TextItem) {
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

        let font_name = self.write_font(&text.font);

        let tts = ts.post_scale(1. / self.pixel_per_pt, 1. / self.pixel_per_pt);
        self.content.items.push(content::TextItem {
            str: glyph_chars,
            // todo: real direction of the text
            dir: match text.lang.dir() {
                Dir::LTR => "ltr".to_string(),
                Dir::RTL => "rtl".to_string(),
                Dir::TTB => "ttb".to_string(),
                Dir::BTT => "btt".to_string(),
            },
            width: x,
            // todo: real height
            height: text.size.to_f32(),
            transform: [
                text.size.to_f32(),
                tts.ky,
                tts.kx,
                text.size.to_f32(),
                tts.tx,
                self.raw_height - tts.ty,
            ],
            font_name,
            has_eol: false,
        });
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
            let mut builder = SvgPath2DBuilder(String::new());
            self.canvas.begin_path();
            // todo: handling no such glyph
            face.outline_glyph(id, &mut builder)?;

            let Paint::Solid(color) = text.fill;
            let c = color.to_rgba();
            let fill_style = format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a);
            self.canvas.set_fill_style(&fill_style.into());
            self.canvas
                .fill_with_path_2d(&Path2d::new_with_path_string(&builder.0).unwrap());
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

    /// Render a geometrical shape into the canvas.
    fn render_shape(
        &mut self,
        ts: sk::Transform,
        mask: Option<&sk::Mask>,
        shape: &Shape,
    ) -> Option<()> {
        let mut builder = SvgPath2DBuilder(String::new());

        // to ensure that our shape focus on the original point
        builder.move_to(0., 0.);
        match shape.geometry {
            Geometry::Line(target) => {
                builder.line_to(target.x.to_f32(), target.y.to_f32());
            }
            Geometry::Rect(size) => {
                let w = size.x.to_f32();
                let h = size.y.to_f32();
                builder.line_to(0., h);
                builder.line_to(w, h);
                builder.line_to(w, 0.);
                builder.close();
            }
            Geometry::Path(ref path) => {
                for elem in &path.0 {
                    match elem {
                        PathItem::MoveTo(p) => {
                            builder.move_to(p.x.to_f32(), p.y.to_f32());
                        }
                        PathItem::LineTo(p) => {
                            builder.line_to(p.x.to_f32(), p.y.to_f32());
                        }
                        PathItem::CubicTo(p1, p2, p3) => {
                            builder.curve_to(
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
            }
        };

        // todo: anti_alias
        if let Some(fill) = &shape.fill {
            let state_guard = CanvasStateGuard(self.canvas);

            let Paint::Solid(color) = fill;
            let c = color.to_rgba();
            let fill_style = format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a);

            #[cfg(feature = "debug_shape_fill")]
            console_log!(
                "fill pure background {} -> {} [{:?}]",
                builder.0,
                fill_style,
                ts
            );

            self.canvas.set_fill_style(&fill_style.into());
            self.canvas.reset_transform().unwrap();
            self.sync_transform(ts);

            self.canvas
                .fill_with_path_2d(&Path2d::new_with_path_string(&builder.0).unwrap());

            drop(state_guard)
        } else if let Some(Stroke {
            paint,
            thickness,
            line_cap,
            line_join,
            dash_pattern,
            miter_limit,
        }) = &shape.stroke
        {
            // todo: dash pattern
            // dash_pattern.as_ref().and_then(|pattern| {
            // tiny-skia only allows dash patterns with an even number of elements,
            // while pdf allows any number.
            // let len = if pattern.array.len() % 2 == 1 {
            //     pattern.array.len() * 2
            // } else {
            //     pattern.array.len()
            // };
            // let dash_array = pattern
            //     .array
            //     .iter()
            //     .map(|l| l.to_f32())
            //     .cycle()
            //     .take(len)
            //     .collect();

            // sk::StrokeDash::new(dash_array, pattern.phase.to_f32())
            //     panic!("dash_pattern");

            // });

            let state_guard = CanvasStateGuard(self.canvas);
            self.canvas.set_line_width(thickness.to_pt());
            self.canvas.set_line_cap(match line_cap {
                geom::LineCap::Butt => "butt",
                geom::LineCap::Round => "round",
                geom::LineCap::Square => "square",
            });
            self.canvas.set_line_join(match line_join {
                geom::LineJoin::Miter => "miter",
                geom::LineJoin::Bevel => "bevel",
                geom::LineJoin::Round => "round",
            });
            self.canvas.set_miter_limit(miter_limit.0);

            let Paint::Solid(color) = paint;
            let c = color.to_rgba();
            let fill_style = format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a);
            self.canvas.set_stroke_style(&fill_style.into());

            // todo: ts, mask
            self.canvas.reset_transform().unwrap();
            self.sync_transform(ts);

            self.canvas
                .stroke_with_path(&Path2d::new_with_path_string(&builder.0).unwrap());

            drop(state_guard)
        }

        Some(())
    }

    /// Render a raster or SVG image into the canvas.
    fn render_image(
        &mut self,
        ts: sk::Transform,
        mask: Option<&sk::Mask>,
        image: &Image,
        size: Size,
    ) -> Option<()> {
        let view_width = size.x.to_f32();
        let view_height = size.y.to_f32();

        let aspect = (image.width() as f32) / (image.height() as f32);
        let scale = ts.sx.max(ts.sy);
        let w = (scale * view_width.max(aspect * view_height)).ceil() as u32;
        let h = ((w as f32) / aspect).ceil() as u32;

        let window = web_sys::window().unwrap();

        let img = window
            .document()
            .unwrap()
            .create_element("img")
            .unwrap()
            .dyn_into::<web_sys::HtmlImageElement>()
            .unwrap();

        let u = js_sys::Uint8Array::new_with_length(image.data().len() as u32);
        u.copy_from(image.data());

        let parts = js_sys::Array::new();
        parts.push(&u);
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &parts,
            web_sys::BlobPropertyBag::new().type_(match image.format() {
                ImageFormat::Raster(e) => match e {
                    RasterFormat::Jpg => "image/jpeg",
                    RasterFormat::Png => "image/png",
                    RasterFormat::Gif => "image/gif",
                },
                ImageFormat::Vector(e) => match e {
                    VectorFormat::Svg => "image/svg+xml",
                },
            }),
        )
        .unwrap();

        let data_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        let remote_data_url = data_url.clone();

        let session_id = self.session_id.clone();

        let session_id2 = session_id.clone();
        let data_url2 = data_url.clone();

        let x = ts.tx;
        let y = ts.ty;

        let img_ref = img.clone();

        let a = Closure::<dyn Fn()>::new(move || {
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .query_selector(format!("canvas[data-typst-session='{}']", session_id).as_str())
                .unwrap();

            // console_log!("loaded {} {}", session_id, remote_data_url);

            let canvas = if let Some(canvas) = canvas {
                canvas
            } else {
                web_sys::Url::revoke_object_url(&remote_data_url).unwrap();
                return;
            };

            let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            // console_log!(
            //     "ready {} {} {:?}",
            //     session_id,
            //     remote_data_url,
            //     (x, y, w, h)
            // );

            let state = CanvasStateGuard(&ctx);
            ctx.reset_transform().unwrap();
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                &img_ref, x as f64, y as f64, w as f64, h as f64,
            )
            .unwrap();
            drop(state);
        });

        img.set_onload(Some(a.as_ref().unchecked_ref()));
        a.forget();

        let a = Closure::<dyn Fn(JsValue)>::new(move |e: JsValue| {
            // console_log!(
            //     "err image loading {} {:?} {:?} {}",
            //     session_id2,
            //     Reflect::get(&e, &"type".into()).unwrap(),
            //     js_sys::JSON::stringify(&e).unwrap(),
            //     data_url2,
            // );
        });

        img.set_onerror(Some(a.as_ref().unchecked_ref()));
        a.forget();

        img.set_src(&data_url);

        Some(())
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

struct SvgPath2DBuilder(String);

impl ttf_parser::OutlineBuilder for SvgPath2DBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "M {} {} ", x, y).unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "L {} {} ", x, y).unwrap();
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    }

    fn close(&mut self) {
        write!(&mut self.0, "Z ").unwrap();
    }
}

struct CanvasStateGuard<'a>(&'a CanvasRenderingContext2d);

impl<'a> CanvasStateGuard<'a> {
    fn new(context: &'a CanvasRenderingContext2d) -> Self {
        context.save();
        Self(context)
    }
}

impl<'a> Drop for CanvasStateGuard<'a> {
    fn drop(&mut self) {
        self.0.restore();
    }
}
