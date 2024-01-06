//! Lowering Typst Document into SvgItem.

use std::sync::Arc;

use ttf_parser::GlyphId;
use typst::layout::Size;
use typst::text::Font;
use typst::visualize::Image;

use crate::font::GlyphProvider;
use crate::vector::ir::{self, GlyphItem};

/// Lower a glyph into vector item.
pub struct Glyph2VecPass<'a> {
    pub gp: &'a GlyphProvider,

    /// Whether to lower ligature information
    pub lowering_ligature: bool,
}

impl<'a> Glyph2VecPass<'a> {
    pub fn new(gp: &'a GlyphProvider, lowering_ligature: bool) -> Self {
        Self {
            gp,
            lowering_ligature: cfg!(feature = "experimental-ligature") && lowering_ligature,
        }
    }

    pub fn glyph(&self, glyph_item: &GlyphItem) -> Option<GlyphItem> {
        match glyph_item {
            GlyphItem::Raw(font, id) => self.raw_glyph(font, *id),
            GlyphItem::Image(..) | GlyphItem::Outline(..) => Some(glyph_item.clone()),
            GlyphItem::None => Some(GlyphItem::None),
        }
    }

    #[cfg(not(feature = "glyph2vec"))]
    fn raw_glyph(&self, _font: &Font, _id: GlyphId) -> Option<GlyphItem> {
        None
    }
}

#[cfg(feature = "glyph2vec")]
impl<'a> Glyph2VecPass<'a> {
    fn ligature_len(&self, font: &Font, id: GlyphId) -> u8 {
        if !self.lowering_ligature {
            return 0;
        }

        self.gp
            .ligature_glyph(font, id)
            .map(|l| l.len())
            .unwrap_or_default() as u8
    }

    fn raw_glyph(&self, font: &Font, id: GlyphId) -> Option<GlyphItem> {
        self.svg_glyph(font, id)
            .map(GlyphItem::Image)
            .or_else(|| self.bitmap_glyph(font, id).map(GlyphItem::Image))
            .or_else(|| self.outline_glyph(font, id).map(GlyphItem::Outline))
    }

    /// Lower an SVG glyph into svg item.
    /// More information: https://learn.microsoft.com/zh-cn/typography/opentype/spec/svg
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ir::ImageGlyphItem>> {
        use crate::vector::ir::Scalar;
        use crate::vector::utils::AbsExt;

        let image = Self::extract_svg_glyph(self.gp, font, id)?;

        // position our image
        let ascender = font
            .metrics()
            .ascender
            .at(typst::layout::Abs::raw(font.metrics().units_per_em))
            .to_f32();

        Some(Arc::new(ir::ImageGlyphItem {
            ts: ir::Transform {
                sx: Scalar(1.),
                ky: Scalar(0.),
                kx: Scalar(0.),
                sy: Scalar(-1.),
                tx: Scalar(0.),
                ty: Scalar(ascender),
            },
            image,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower a bitmap glyph into the svg text.
    fn bitmap_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ir::ImageGlyphItem>> {
        use crate::vector::utils::AbsExt;
        /// Use types from `tiny-skia` crate.
        use tiny_skia as sk;

        let ppem = u16::MAX;
        let upem = font.metrics().units_per_em as f32;

        let (glyph_image, raster_x, raster_y) = self.gp.bitmap_glyph(font, id, ppem)?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?

        let w = glyph_image.width() as f64;
        let h = glyph_image.height() as f64;
        let sz = Size::new(typst::layout::Abs::raw(w), typst::layout::Abs::raw(h));

        let image = ir::ImageItem {
            image: Arc::new(glyph_image.into()),
            size: sz.into(),
        };

        // position our image
        // first, the ascender is used
        // next, also apply an offset of (1 - ascender) like typst
        let adjusted = font.metrics().ascender * 2. - typst::layout::Em::one();
        // let adjusted = font.metrics().ascender;

        let adjusted = adjusted
            .at(typst::layout::Abs::raw(font.metrics().units_per_em))
            .to_f32();

        let ts = sk::Transform::from_scale(upem / w as f32, -upem / h as f32);

        // size
        let dx = raster_x as f32;
        let dy = raster_y as f32;
        let ts = ts.post_translate(dx, adjusted + dy);

        Some(Arc::new(ir::ImageGlyphItem {
            ts: ts.into(),
            image,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower an outline glyph into svg text. This is the "normal" case.
    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ir::OutlineGlyphItem>> {
        let d = self.gp.outline_glyph(font, id)?.into();

        Some(Arc::new(ir::OutlineGlyphItem {
            ts: None,
            d,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    fn extract_svg_glyph(g: &GlyphProvider, font: &Font, id: GlyphId) -> Option<ir::ImageItem> {
        struct FindViewBoxResult<'a> {
            start_span: Option<xmlparser::StrSpan<'a>>,
            first_viewbox: Option<(xmlparser::StrSpan<'a>, xmlparser::StrSpan<'a>)>,
        }

        /// Find the string location of the **first** viewBox attribute.
        /// When there are multiple viewBox attributes, the first one is used
        /// (as many xml-based dom engines do).
        fn find_viewbox_attr(svg_str: &'_ str) -> FindViewBoxResult<'_> {
            let document = xmlparser::Tokenizer::from(svg_str);

            let mut start_span = None;
            let mut first_viewbox = None;
            for n in document {
                let tok = n.unwrap();
                match tok {
                    xmlparser::Token::ElementStart { span, local, .. } => {
                        if local.as_str() == "svg" {
                            start_span = Some(span);
                        }
                    }
                    xmlparser::Token::Attribute {
                        span, local, value, ..
                    } => {
                        if local.as_str() == "viewBox" {
                            first_viewbox = Some((span, value));
                            break;
                        }
                    }
                    xmlparser::Token::ElementEnd { .. } => break,
                    _ => {}
                }
            }

            FindViewBoxResult {
                start_span,
                first_viewbox,
            }
        }
        use crate::vector::utils::AbsExt;
        use std::io::Read;

        use once_cell::sync::OnceCell;

        static WARN_VIEW_BOX: OnceCell<()> = OnceCell::new();

        let data = g.svg_glyph(font, id)?;
        let mut data = data.as_ref();

        let font_metrics = font.metrics();

        // Decompress SVGZ.
        let mut decoded = vec![];

        // The first three bytes of the gzip-encoded document header must be
        //   0x1F, 0x8B, 0x08.
        if data.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = flate2::read::GzDecoder::new(data);
            decoder.read_to_end(&mut decoded).ok()?;
            data = &decoded;
        }

        // todo: It is also legal to provide a SVG document containing multiple glyphs.
        // > When a font engine renders glyph 14, the result shall be the same as
        // > rendering the following SVG document:
        // > `  <svg> <defs> <use #glyph{id}> </svg>`
        // See: <https://learn.microsoft.com/en-us/typography/opentype/spec/svg#glyph-identifiers>

        let upem = typst::layout::Abs::raw(font.units_per_em());
        let (width, height) = (upem.to_f32(), upem.to_f32());
        let origin_ascender = font_metrics.ascender.at(upem).to_f32();

        let doc_string = String::from_utf8(data.to_owned()).unwrap();

        // todo: verify SVG capability requirements and restrictions

        // Partially parse the view box attribute
        let mut svg_str = std::str::from_utf8(data).ok()?.to_owned();
        let FindViewBoxResult {
            start_span,
            first_viewbox,
        } = find_viewbox_attr(svg_str.as_str());

        // determine view box
        let view_box = first_viewbox
            .as_ref()
            .map(|s| {
                WARN_VIEW_BOX.get_or_init(|| {
                    println!(
                        "render_svg_glyph with viewBox, This should be helpful if you can help us verify the result: {:?} {:?}",
                        font.info().family,
                        doc_string
                    );
                });
                s.1.as_str().to_owned()
            })
            .unwrap_or_else(|| format!("0 {} {width} {height}", -origin_ascender));

        // determine view box
        match first_viewbox {
            Some((span, ..)) => {
                // replace the first viewBox attribute
                svg_str.replace_range(span.range(), format!(r#"viewBox="{view_box}""#).as_str());
            }
            None => {
                // insert viewBox attribute to the begin of svg tag
                svg_str.insert_str(
                    start_span.unwrap().range().end,
                    format!(r#" viewBox="{view_box}""#).as_str(),
                );
            }
        }

        let glyph_image = typst::visualize::Image::new(
            svg_str.as_bytes().to_vec().into(),
            typst::visualize::ImageFormat::Vector(typst::visualize::VectorFormat::Svg),
            // typst::geom::Axes::new(width as u32, height as u32),
            None,
        )
        .ok()?;

        let sz = Size::new(
            typst::layout::Abs::raw(glyph_image.width() as f64),
            typst::layout::Abs::raw(glyph_image.height() as f64),
        );

        Some(ir::ImageItem {
            image: Arc::new(glyph_image.into()),
            size: sz.into(),
        })
    }
}

/// Lower a raster or SVG image into svg item.
#[comemo::memoize]
fn lower_image(image: &Image, size: Size) -> ir::ImageItem {
    ir::ImageItem {
        image: Arc::new(image.clone().into()),
        size: size.into(),
    }
}