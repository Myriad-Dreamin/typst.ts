use std::{io::Read, ops::Deref};

use base64::Engine;
use once_cell::sync::OnceCell;
use typst::{
    geom::{Abs, Axes, Size},
    image::{Image, ImageFormat, RasterFormat, VectorFormat},
};
use typst_ts_core::{error::prelude::*, font::GlyphProvider};

use ttf_parser::GlyphId;
use typst::font::Font;

use crate::{
    ir::{AbsoulteRef, FlatTextItem, GlyphItem, StyleNs},
    ir::{FlatSvgItem, GroupRef, Module, TransformItem},
    ir::{PathItem, PathStyle},
    sk,
    utils::{console_log, AbsExt, PerfEvent, ToCssExt},
    ClipPathMap, DefaultExportFeature, ExportFeature, StyleDefMap,
};

static WARN_VIEW_BOX: OnceCell<()> = OnceCell::new();

pub struct SvgRenderTask<'m, 't, Feat: ExportFeature = DefaultExportFeature> {
    pub glyph_provider: GlyphProvider,

    pub module: &'m Module,

    pub style_defs: &'t mut StyleDefMap,
    pub clip_paths: &'t mut ClipPathMap,

    pub page_off: usize,
    pub render_text_element: bool,

    // errors: Vec<Error>,
    pub _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<'m, 't, Feat: ExportFeature> SvgRenderTask<'m, 't, Feat> {
    #[inline]
    fn perf_event(&self, _name: &'static str) -> Option<PerfEvent> {
        None
    }

    /// Render an item into the a `<g/>` element.
    pub fn render_item(&mut self, abs_ref: AbsoulteRef) -> ZResult<String> {
        let item = self.module.get_item(&abs_ref).ok_or_else(|| {
            error_once!(
                "SvgRenderTask.ItemNotFound",
                abs_ref: format!("{:?}", abs_ref)
            )
        })?;
        self.render_item_inner(abs_ref, item)
    }

    pub fn render_item_inner(
        &mut self,
        abs_ref: AbsoulteRef,
        item: &FlatSvgItem,
    ) -> ZResult<String> {
        match item.deref() {
            FlatSvgItem::Group(group) => self.render_group(abs_ref, group),
            FlatSvgItem::Text(text) => self.render_text(abs_ref, text),
            FlatSvgItem::Path(path) => Ok(format!(
                r#"<g data-tid="{}">{}</g>"#,
                abs_ref.as_svg_id("p"),
                self.render_path(path)?,
            )),
            FlatSvgItem::Item(transformed) => {
                let item = self.render_item(abs_ref.id.make_absolute_ref(transformed.1.clone()))?;
                Ok(format!(
                    r#"<g data-tid="{}" {}>{}</g>"#,
                    abs_ref.as_svg_id("p"),
                    self.get_css(&transformed.0),
                    item
                ))
            }
            FlatSvgItem::Link(link) => Ok(format!(
                r#"<g data-tid="{}"><a xlink:href="{}" target="_blank"><rect class="pseudo-link" width="{}" height="{}"></rect></a></g>"#,
                abs_ref.as_svg_id("l"),
                link.href.replace('&', "&amp;"),
                link.size.x.to_pt(),
                link.size.y.to_pt(),
            )),
            FlatSvgItem::Image(image) => Ok(format!(
                r#"<g data-tid="{}">{}</g>"#,
                abs_ref.as_svg_id("i"),
                Self::render_image(&image.image, image.size)?,
            )),
            FlatSvgItem::Glyph(_) | FlatSvgItem::None => {
                panic!("SvgRenderTask.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }

    /// Render a frame into svg text.
    fn render_group(&mut self, abs_ref: AbsoulteRef, group: &GroupRef) -> ZResult<String> {
        let mut g = vec![format!(
            r#"<g class="group" data-tid="{}">"#,
            abs_ref.as_svg_id("p")
        )];
        let mut normal_g = vec![];
        let mut link_g = vec![];

        for (pos, item) in group.0.iter() {
            let def_id = abs_ref.id.make_absolute_ref(item.clone());
            let item = self.module.get_item(&def_id).ok_or_else(|| {
                error_once!(
                    "SvgRenderTask.ItemNotFound",
                    abs_ref: format!("{:?}", abs_ref)
                )
            })?;

            let g = if let FlatSvgItem::Link(_) = item.deref() {
                &mut link_g
            } else {
                &mut normal_g
            };

            g.push(format!(
                r#"<g transform="translate({:.3},{:.3})" >"#,
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
            TransformItem::Translate(t) => {
                format!("translate({:.3},{:.3})", t.x.to_f32(), t.y.to_f32())
            }
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

    /// Render a geometrical shape into svg text.
    fn render_path(&mut self, path: &PathItem) -> ZResult<String> {
        render_path(path)
    }

    /// Render a text run into the svg text.
    pub(crate) fn render_text(
        &mut self,
        abs_ref: AbsoulteRef,
        text: &FlatTextItem,
    ) -> ZResult<String> {
        let _r = self.perf_event("render_text");

        let mut text_list = vec![];
        let shape = &text.shape;

        // Scale is in pixel per em, but curve data is in font design units, so
        // we have to divide by units per em.
        let upem = shape.upem.0 as f32;
        let ppem = shape.ppem.0 as f32;
        let ascender = shape.ascender.to_f32();

        let fill = if shape.fill.as_ref() == "#000" {
            r#"tb"#.to_owned()
        } else {
            let fill_id = format!(r#"f{}"#, shape.fill.trim_start_matches('#'));
            let fill_key = (StyleNs::Fill, shape.fill.clone());
            self.style_defs.entry(fill_key).or_insert_with(|| {
                format!(r#"g.{} {{ --glyph_fill: {}; }} "#, fill_id, shape.fill)
            });

            fill_id
        };
        text_list.push(format!(
            r#"<g class="typst-txt {}" data-tid="{}">"#,
            fill,
            abs_ref.as_svg_id("p")
        ));
        text_list.push(format!(r#"<g transform="scale({},{})">"#, ppem, -ppem));

        let mut x = 0f32;
        for (offset, advance, glyph) in text.content.glyphs.iter() {
            let offset = x + offset.to_f32();
            let ts = offset / ppem;
            let adjusted = (ts * 2.).round() / 2.;

            let glyph_id = glyph.as_svg_id("g");
            let e = format!(r##"<use href="#{}"/>"##, glyph_id);

            text_list.push(format!(
                r#"<g transform="translate({},0)">{}</g>"#,
                adjusted, e
            ));

            x += advance.to_f32();
        }

        text_list.push("</g>".to_string());
        if self.render_text_element {
            // text_list.push(format!(
            //     r#"<h5:span textLength="{}" font-size="{}" class="tsel">{}</h5:span>"#,
            //     v,
            //     upem * ppem,
            //     xml::escape::escape_str_pcdata(&text.content.content),
            // ));

            // todo: investigate &nbsp;
            text_list.push(format!(
                r#"<foreignObject x="0" y="-{}" width="{}" height="{}"><h5:div class="tsel" style="font-size: {}px;">{}</h5:div></foreignObject>"#,
                ascender,
                x,
                upem * ppem,
                upem * ppem,
                xml::escape::escape_str_pcdata(&text.content.content)
            ));
        }
        text_list.push("</g>".to_string());

        Ok(text_list.join(""))
    }

    pub fn render_glyph(&mut self, glyph: &AbsoulteRef, glyph_item: &GlyphItem) -> Option<String> {
        let gp = &self.glyph_provider;
        Self::render_glyph_inner(gp, glyph, glyph_item)
    }

    #[comemo::memoize]
    fn render_glyph_inner(
        gp: &GlyphProvider,
        glyph: &AbsoulteRef,
        glyph_item: &GlyphItem,
    ) -> Option<String> {
        match glyph_item {
            GlyphItem::Raw(font, id) => {
                let id = *id;
                // todo: server side render
                Self::render_svg_glyph(gp, font, glyph, id)
                    .or_else(|| Self::render_bitmap_glyph(gp, font, glyph, id))
                    .or_else(|| Self::render_outline_glyph(gp, font, glyph, id))
            }
        }
    }

    /// Render an SVG glyph into the svg text.
    /// More information: https://learn.microsoft.com/zh-cn/typography/opentype/spec/svg
    fn render_svg_glyph(
        glyph_provider: &GlyphProvider,
        font: &Font,
        glyph: &AbsoulteRef,
        id: GlyphId,
    ) -> Option<String> {
        let glyph_image = extract_svg_glyph(glyph_provider, font, id)?;

        // position our image
        let ascender = font
            .metrics()
            .ascender
            .at(Abs::raw(font.metrics().units_per_em))
            .to_f32();

        let img = Self::render_image(
            &glyph_image,
            Size::new(
                Abs::pt(glyph_image.width() as f64),
                Abs::pt(glyph_image.height() as f64),
            ),
        )
        .ok()?;

        let glyph_id = glyph.as_svg_id("g");
        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="svg_glyph"><g transform="scale(1, -1), translate(0,{})">{}</g></symbol>"#,
            glyph_id, -ascender, img
        );
        Some(symbol_def)
    }

    /// Render a bitmap glyph into the svg text.
    fn render_bitmap_glyph(
        glyph_provider: &GlyphProvider,
        font: &Font,
        glyph: &AbsoulteRef,
        id: GlyphId,
    ) -> Option<String> {
        let ppem = u16::MAX;
        let upem = font.metrics().units_per_em as f32;

        let (glyph_image, raster_x, raster_y) = glyph_provider.bitmap_glyph(font, id, ppem)?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?

        // position our image
        let ascender = font
            .metrics()
            .ascender
            .at(Abs::raw(font.metrics().units_per_em))
            .to_f32();

        let sy = upem / glyph_image.height() as f32;

        let ts = sk::Transform::from_scale(upem / glyph_image.width() as f32, -sy);

        // size
        let dx = raster_x as f32;
        let dy = raster_y as f32;
        let ts = ts.post_translate(dx, ascender + dy);

        let img = Self::render_image(
            &glyph_image,
            Size::new(
                Abs::pt(glyph_image.width() as f64),
                Abs::pt(glyph_image.height() as f64),
            ),
        )
        .ok()?;

        let glyph_id = glyph.as_svg_id("g");
        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="bitmap_glyph"><g transform="{}">{}</g></symbol>"#,
            glyph_id,
            ts.to_css(),
            img
        );
        Some(symbol_def)
    }

    /// Render an outline glyph into svg text. This is the "normal" case.
    fn render_outline_glyph(
        glyph_provider: &GlyphProvider,
        font: &Font,
        glyph: &AbsoulteRef,
        id: GlyphId,
    ) -> Option<String> {
        if cfg!(feature = "debug_glyph_render") {
            console_log!("render_outline_glyph: {:?}", font.info());
        }

        let d = glyph_provider.outline_glyph(font, id)?;

        let glyph_id = glyph.as_svg_id("g");
        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="outline_glyph"><path d="{}"/></symbol>"#,
            glyph_id, d
        );
        Some(symbol_def)
    }

    /// Render a raster or SVG image into svg text.
    // todo: error handling
    #[comemo::memoize]
    pub fn render_image(image: &Image, size: Size) -> ZResult<String> {
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

fn extract_svg_glyph(g: &GlyphProvider, font: &Font, id: GlyphId) -> Option<Image> {
    let data = g.svg_glyph(font, id)?;
    let mut data = data.as_ref();

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

    // todo: verify SVG capability requirements and restrictions

    // Parse XML.
    let mut svg_str = std::str::from_utf8(data).ok()?.to_owned();
    let document = xmlparser::Tokenizer::from(svg_str.as_str());
    let mut start_span = None;
    let mut last_viewbox = None;
    for n in document {
        let tok = n.unwrap();
        match tok {
            xmlparser::Token::ElementStart { span, local, .. } => {
                if local.as_str() == "svg" {
                    start_span = Some(span);
                    break;
                }
            }
            xmlparser::Token::Attribute {
                span, local, value, ..
            } => {
                if local.as_str() == "viewBox" {
                    last_viewbox = Some((span, value));
                }
            }
            xmlparser::Token::ElementEnd { .. } => break,
            _ => {}
        }
    }

    // update view box
    let view_box = last_viewbox.as_ref()
        .map(|s| {
            WARN_VIEW_BOX.get_or_init(|| {
                console_log!(
                    "render_svg_glyph with viewBox, This should be helpful if you can help us verify the result: {:?} {:?}",
                    font.info().family,
                    doc_string
                );
            });
            s.1.as_str().to_owned()
        })
        .unwrap_or_else(|| format!("0 {} {} {}", -origin_ascender, width, height));

    match last_viewbox {
        Some((span, ..)) => {
            svg_str.replace_range(span.range(), format!(r#"viewBox="{}""#, view_box).as_str());
        }
        None => {
            svg_str.insert_str(
                start_span.unwrap().range().end,
                format!(r#" viewBox="{}""#, view_box).as_str(),
            );
        }
    }

    if cfg!(feature = "debug_glyph_render") {
        console_log!(
            "render_svg_glyph prepared document: {:?} {:?}",
            font.info().family,
            svg_str
        );
    }

    let image = Image::new_raw(
        svg_str.as_bytes().to_vec().into(),
        typst::image::ImageFormat::Vector(typst::image::VectorFormat::Svg),
        Axes::new(width as u32, height as u32),
        None,
    )
    .ok()?;

    Some(image)
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

#[comemo::memoize]
fn render_path(path: &PathItem) -> ZResult<String> {
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
