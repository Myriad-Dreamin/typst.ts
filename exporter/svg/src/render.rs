use std::{collections::HashMap, sync::Arc};

use base64::Engine;
use typst::{
    font::FontInfo,
    geom::Size,
    image::{Image, ImageFormat, RasterFormat, VectorFormat},
};
use typst_ts_core::{
    annotation::AnnotationList, error::prelude::*, font::GlyphProvider, TextContent,
};

use ttf_parser::GlyphId;
use typst::font::Font;

use crate::{
    ir::{DefId, FlatSvgItem, GroupRef, Module, TransformItem},
    ir::{FlatTextItem, GlyphItem, StyleNs},
    ir::{PathItem, PathStyle},
    utils::{console_log, AbsExt, PerfEvent},
    DefaultRenderFeature, RenderFeature,
};

pub struct SvgRenderTask<'m, 't, Feat: RenderFeature = DefaultRenderFeature> {
    pub glyph_provider: GlyphProvider,

    pub annotations: &'t mut AnnotationList,
    pub module: &'m Module,
    pub text_content: &'t mut TextContent,

    pub style_defs: &'t mut HashMap<(StyleNs, Arc<str>), (String, u32)>,
    pub glyph_defs: &'t mut HashMap<String, (String, u32)>,
    pub clip_paths: &'t mut HashMap<String, u32>,

    pub page_off: usize,
    pub width_px: u32,
    pub height_px: u32,
    pub raw_height: f32,

    pub font_map: HashMap<FontInfo, u32>,

    // errors: Vec<Error>,
    pub _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<'m, 't, Feat: RenderFeature> SvgRenderTask<'m, 't, Feat> {
    #[inline]
    fn perf_event(&self, _name: &'static str) -> Option<PerfEvent> {
        None
    }

    pub fn render_item(&mut self, def_id: DefId) -> ZResult<String> {
        let item = self
            .module
            .get_item(def_id)
            .ok_or_else(|| error_once!("SvgRenderTask.ItemNotFound", def_id: def_id.0))?;
        self.render_item_inner(def_id, item)
    }

    pub fn render_item_inner(&mut self, def_id: DefId, item: &FlatSvgItem) -> ZResult<String> {
        match item {
            FlatSvgItem::Group(group) => self.render_group(def_id, group),
            FlatSvgItem::Text(text) => self.render_text(text),
            FlatSvgItem::Path(path) => self.render_path(path),
            FlatSvgItem::Item(transformed) => {
                let item = self.render_item(def_id.make_absolute(transformed.1))?;
                Ok(format!(
                    r#"<g {}>{}</g>"#,
                    self.get_css(&transformed.0),
                    item
                ))
            }
            FlatSvgItem::Link(link) => Ok(format!(
                r#"<a xlink:href="{}" target="_blank"><rect class="pseudo-link" width="{}" height="{}"></rect></a>"#,
                link.href.replace('&', "&amp;"),
                link.size.x.to_pt(),
                link.size.y.to_pt(),
            )),
            FlatSvgItem::Image(image) => self.render_image(&image.image, image.size),
            FlatSvgItem::Glyph(_) | FlatSvgItem::None => {
                panic!("SvgRenderTask.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }

    /// Render a frame into the canvas.
    fn render_group(&mut self, group_id: DefId, group: &GroupRef) -> ZResult<String> {
        let mut g = vec![format!(r#"<g class="group">"#)];
        let mut normal_g = vec![];
        let mut link_g = vec![];

        for (pos, item) in group.0.iter() {
            let def_id = group_id.make_absolute(*item);
            let item = self
                .module
                .get_item(def_id)
                .ok_or_else(|| error_once!("SvgRenderTask.ItemNotFound", def_id: def_id.0))?;

            let g = if let FlatSvgItem::Link(_) = item {
                &mut link_g
            } else {
                &mut normal_g
            };

            g.push(format!(
                r#"<g transform="translate({},{})" >"#,
                pos.x.to_pt(),
                pos.y.to_pt()
            ));
            g.push(self.render_item_inner(def_id, item)?);
            g.push("</g>".to_owned());
        }

        g.append(&mut normal_g);
        g.append(&mut link_g);

        g.push("</g>".to_owned());

        Ok(g.join(""))
    }

    fn get_css(&mut self, transform: &TransformItem) -> String {
        match transform {
            TransformItem::Matrix(m) => {
                format!(
                    r#"transform="matrix({},{},{},{},{},{})""#,
                    m.sx.get(),
                    m.ky.get(),
                    m.kx.get(),
                    m.sy.get(),
                    m.tx.to_pt(),
                    m.ty.to_pt()
                )
            }
            TransformItem::Translate(t) => format!("translate({},{})", t.x.to_f32(), t.y.to_f32()),
            TransformItem::Scale(s) => format!("scale({},{})", s.0.get(), s.1.get()),
            TransformItem::Rotate(angle) => format!("rotate({})", angle.0),
            TransformItem::Skew(angle) => {
                format!("skewX({}) skewY({})", angle.0.get(), angle.1.get())
            }
            TransformItem::Clip(c) => {
                let clip_id;
                if let Some(c) = self.clip_paths.get(&c.d) {
                    clip_id = *c;
                } else {
                    let cid = self.clip_paths.len() as u32;
                    self.clip_paths.insert(c.d.clone(), cid);
                    clip_id = cid;
                }

                format!(r##"clip-path="url(#c{:x})""##, clip_id)
            }
        }
    }

    /// Render a geometrical shape into the canvas.
    fn render_path(&mut self, path: &PathItem) -> ZResult<String> {
        let mut p = vec!["<path ".to_owned()];
        p.push(format!(r#"d="{}" "#, path.d));
        for style in &path.styles {
            match style {
                PathStyle::Fill(color) => {
                    p.push(format!(r#"fill="{}" "#, color));
                }
                PathStyle::Stroke(color) => {
                    p.push(format!(r#"stroke="{}" "#, color));
                }
                PathStyle::StrokeWidth(width) => {
                    p.push(format!(r#"stroke-width="{}" "#, width.to_f32()));
                }
                PathStyle::StrokeLineCap(cap) => {
                    p.push(format!(r#"stroke-linecap="{}" "#, cap));
                }
                PathStyle::StrokeLineJoin(join) => {
                    p.push(format!(r#"stroke-linejoin="{}" "#, join));
                }
                PathStyle::StrokeMitterLimit(limit) => {
                    p.push(format!(r#"stroke-miterlimit="{}" "#, limit.0));
                }
                PathStyle::StrokeDashArray(array) => {
                    p.push(r#"stroke-dasharray="#.to_owned());
                    for (i, v) in array.iter().enumerate() {
                        if i > 0 {
                            p.push(" ".to_owned());
                        }
                        p.push(format!("{}", v.to_f32()));
                    }
                    p.push(r#"" "#.to_owned());
                }
                PathStyle::StrokeDashOffset(offset) => {
                    p.push(format!(r#"stroke-dashoffset="{}" "#, offset.to_f32()));
                }
            }
        }
        p.push("/>".to_owned());
        let p = p.join("");
        Ok(p)
    }

    /// Render a text run into the self.canvas.
    pub(crate) fn render_text(&mut self, text: &FlatTextItem) -> ZResult<String> {
        let _r = self.perf_event("render_text");

        let mut text_list = vec![];
        let shape = &text.shape;

        let fill = if shape.fill.as_ref() == "rgba(0, 0, 0, 255)" {
            r#"tb"#.to_owned()
        } else {
            // format!(r#" fill="{}" "#, shape.fill);
            let fill_key = (StyleNs::Fill, shape.fill.clone());
            let fill_id;
            if let Some(idx) = self.style_defs.get(&fill_key) {
                fill_id = idx.1;
            } else {
                let new_id = self.style_defs.len() as u32;
                let fill_style = format!(r#"g.f{:x} {{ --glyph_fill: {}; }} "#, new_id, shape.fill);
                self.style_defs.insert(fill_key, (fill_style, new_id));
                fill_id = new_id;
            };

            format!(r#"f{:x}"#, fill_id)
        };
        text_list.push(format!(
            r#"<g class="t {}" transform="scale({},{})">"#,
            fill, shape.ppem, -shape.ppem
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
                        .or_else(|| self.render_outline_glyph(ts, &font, id));
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
    fn render_outline_glyph(&mut self, x: f32, font: &Font, id: GlyphId) -> Option<String> {
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
                r#"<symbol overflow="visible" id="g{:x}" class="outline_glyph">{}</symbol>"#,
                new_id, glyph_data
            );
            self.glyph_defs.insert(glyph_data, (symbol_def, new_id));
            glyph_id = new_id;
        }

        Some(format!(r##"<use href="#g{:x}" x="{:.1}"/>"##, glyph_id, x))
    }

    /// Render a raster or SVG image into the canvas.
    // todo: error handling
    pub(crate) fn render_image(&mut self, image: &Image, size: Size) -> ZResult<String> {
        let _r = self.perf_event("render_image");

        let _l = self.perf_event("load_image");
        let image_url = rasterize_embedded_image_url(image).unwrap();

        // resize image to fit the view
        let size = size;
        let view_width = size.x.to_f32();
        let view_height = size.y.to_f32();

        let aspect = (image.width() as f32) / (image.height() as f32);

        let w = view_width.max(aspect * view_height);
        let h = w / aspect;
        Ok(format!(
            r#"<image x="0" y="0" width="{}" height="{}" xlink:href="{}" />"#,
            w, h, image_url
        ))
    }
}

#[comemo::memoize]
fn extract_outline_glyph(g: GlyphProvider, font: &Font, id: GlyphId) -> Option<String> {
    let d = g.outline_glyph(font, id)?;
    Some(format!(r#"<path d="{}"/>"#, d))
}

#[derive(Debug, Clone)]
struct ImageUrl(String);

#[cfg(feature = "web")]
impl Drop for ImageUrl {
    fn drop(&mut self) {
        web_sys::Url::revoke_object_url(&self.0).unwrap();
    }
}

#[comemo::memoize]
#[cfg(feature = "web")]
fn rasterize_image_url(image: &Image) -> Option<Arc<ImageUrl>> {
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
                // todo: security check
                // https://security.stackexchange.com/questions/148507/how-to-prevent-xss-in-svg-file-upload
                // todo: use our custom font
                VectorFormat::Svg => "image/svg+xml",
            },
        }),
    )
    .unwrap();

    // todo: memory leak
    let data_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    Some(Arc::new(ImageUrl(data_url)))
}

fn rasterize_embedded_image_url(image: &Image) -> Option<String> {
    let url = match image.format() {
        ImageFormat::Raster(e) => match e {
            RasterFormat::Jpg => "data:image/jpeg;base64,",
            RasterFormat::Png => "data:image/png;base64,",
            RasterFormat::Gif => "data:image/gif;base64,",
        },
        ImageFormat::Vector(e) => match e {
            VectorFormat::Svg => "data:image/svg+xml;base64,",
        },
    };

    let mut data = base64::engine::general_purpose::STANDARD.encode(image.data());
    data.insert_str(0, url);
    Some(data)
}
