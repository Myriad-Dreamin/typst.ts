use base64::Engine;
use typst_ts_core::font::GlyphProvider;

use crate::{
    ir::{self, GlyphItem, Module},
    ir::{AbsoulteRef, Image, ImageGlyphItem, OutlineGlyphItem, Scalar, Size, StyleNs},
    ir::{PathItem, PathStyle},
    lowering::GlyphLowerBuilder,
    utils::ToCssExt,
    vm::{GroupContext, RenderVm},
    ClipPathMap, DefaultExportFeature, ExportFeature, StyleDefMap,
};

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

pub struct RenderGroup<'s, 'm, 't, Feat: ExportFeature> {
    pub t: &'s mut SvgRenderTask<'m, 't, Feat>,
    pub class: Option<String>,
    pub tid: AbsoulteRef,
    pub text_content: Option<String>,
    pub transforms: Vec<String>,
    pub content: Vec<String>,
}

impl<'s, 'm, 't, Feat: ExportFeature> GroupContext for RenderGroup<'s, 'm, 't, Feat> {
    fn transform_matrix(mut self, m: &crate::ir::Transform) -> Self {
        self.transforms.push(format!(
            r#"transform="matrix({},{},{},{},{},{})""#,
            m.sx.0, m.ky.0, m.kx.0, m.sy.0, m.tx.0, m.ty.0
        ));
        self
    }

    fn transform_translate(mut self, matrix: crate::ir::Axes<crate::ir::Abs>) -> Self {
        self.transforms.push(format!(
            r#"transform="translate({:.3},{:.3})""#,
            matrix.x.0, matrix.y.0
        ));
        self
    }

    fn transform_scale(mut self, x: crate::ir::Ratio, y: crate::ir::Ratio) -> Self {
        self.transforms
            .push(format!(r#"transform="scale({},{})""#, x.0, y.0));
        self
    }

    fn transform_rotate(mut self, matrix: Scalar) -> Self {
        self.transforms
            .push(format!(r#"transform="rotate({})""#, matrix.0));
        self
    }

    fn transform_skew(mut self, matrix: (crate::ir::Ratio, crate::ir::Ratio)) -> Self {
        self.transforms.push(format!(
            r#"transform="skewX({}) skewY({})""#,
            matrix.0 .0, matrix.1 .0
        ));
        self
    }

    fn transform_clip(mut self, matrix: &crate::ir::PathItem) -> Self {
        let clip_id;
        if let Some(c) = self.t.clip_paths.get(&matrix.d) {
            clip_id = *c;
        } else {
            let cid = self.t.clip_paths.len() as u32;
            self.t.clip_paths.insert(matrix.d.clone(), cid);
            clip_id = cid;
        }

        self.transforms
            .push(format!(r##"clip-path="url(#c{:x})""##, clip_id));
        self
    }

    fn drop_item_at(&mut self, pos: crate::ir::Point, item: AbsoulteRef) {
        self.content.push(format!(
            r#"<g transform="translate({:.3},{:.3})" >"#,
            pos.x.0, pos.y.0
        ));
        self.content.push(self.t.render_item(item));
        self.content.push("</g>".to_owned());
    }

    fn drop_glyph(&mut self, pos: Scalar, glyph: &AbsoulteRef) {
        let adjusted = (pos.0 * 2.).round() / 2.;

        let glyph_id = glyph.as_svg_id("g");
        let e = format!(r##"<use href="#{}"/>"##, glyph_id);

        self.content.push(format!(
            r#"<g transform="translate({},0)">{}</g>"#,
            adjusted, e
        ));
    }
}

impl<'s, 'm, 't, Feat: ExportFeature> From<RenderGroup<'s, 'm, 't, Feat>> for String {
    fn from(s: RenderGroup<'s, 'm, 't, Feat>) -> Self {
        let (pre_text_content, post_text_content) = if let Some(post) = s.text_content {
            (
                format!("><g {}>", s.transforms.join(" ")),
                format!("</g>{}", post),
            )
        } else {
            (format!(" {}>", s.transforms.join(" ")), "".to_owned())
        };

        let pre_content = if let Some(class) = s.class {
            format!(
                r#"<g class="{}" data-tid="{}"{}"#,
                class,
                s.tid.as_svg_id("p"),
                pre_text_content,
            )
        } else {
            format!(
                r#"<g data-tid="{}"{}"#,
                s.tid.as_svg_id("p"),
                pre_text_content
            )
        };

        pre_content + &s.content.join("") + &post_text_content + "</g>"
    }
}

impl<'m, 't, Feat: ExportFeature> SvgRenderTask<'m, 't, Feat> {
    pub fn render_glyph(&mut self, glyph: &AbsoulteRef, glyph_item: &GlyphItem) -> Option<String> {
        let gp = &self.glyph_provider;
        Self::render_glyph_inner(gp, glyph, glyph_item)
    }

    #[comemo::memoize]
    pub fn render_glyph_pure(glyph: &AbsoulteRef, glyph_item: &GlyphItem) -> Option<String> {
        Self::render_glyph_pure_inner(glyph, glyph_item)
    }

    #[comemo::memoize]
    fn render_glyph_inner(
        gp: &GlyphProvider,
        glyph: &AbsoulteRef,
        glyph_item: &GlyphItem,
    ) -> Option<String> {
        if matches!(glyph_item, GlyphItem::Raw(..)) {
            return Self::render_glyph_pure_inner(
                glyph,
                &GlyphLowerBuilder::new(gp).lower_glyph(glyph_item)?,
            );
        }

        Self::render_glyph_pure_inner(glyph, glyph_item)
    }

    fn render_glyph_pure_inner(glyph: &AbsoulteRef, glyph_item: &GlyphItem) -> Option<String> {
        match glyph_item {
            GlyphItem::Image(image_glyph) => Self::render_image_glyph(glyph, image_glyph),
            GlyphItem::Outline(outline_glyph) => Self::render_outline_glyph(glyph, outline_glyph),
            GlyphItem::Raw(..) => unreachable!(),
        }
    }

    /// Render an image glyph into the svg text.
    fn render_image_glyph(glyph: &AbsoulteRef, ig: &ImageGlyphItem) -> Option<String> {
        let img = render_image(&ig.image.image, ig.image.size);

        let glyph_id = glyph.as_svg_id("g");
        let ts = ig.ts.to_css();
        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="image_glyph"><g transform="{}">{}</g></symbol>"#,
            glyph_id, ts, img
        );
        Some(symbol_def)
    }

    /// Render an outline glyph into svg text. This is the "normal" case.
    fn render_outline_glyph(
        glyph: &AbsoulteRef,
        outline_glyph: &OutlineGlyphItem,
    ) -> Option<String> {
        let glyph_id = glyph.as_svg_id("g");
        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="outline_glyph"><path d="{}"/></symbol>"#,
            glyph_id, outline_glyph.d
        );
        Some(symbol_def)
    }

    #[comemo::memoize]
    fn render_image_inner(abs_ref: AbsoulteRef, image_item: &crate::ir::ImageItem) -> String {
        format!(
            r#"<g data-tid="{}">{}</g>"#,
            abs_ref.as_svg_id("i"),
            render_image(&image_item.image, image_item.size),
        )
    }
}

impl<'s, 'm: 's, 't: 's, Feat: ExportFeature + 's> RenderVm<'s, 'm>
    for SvgRenderTask<'m, 't, Feat>
{
    type Resultant = String;
    type Group = RenderGroup<'s, 'm, 't, Feat>;

    fn get_item(&self, value: &AbsoulteRef) -> Option<&'m crate::ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_group(&'s mut self, v: &AbsoulteRef) -> Self::Group {
        Self::Group {
            t: self,
            tid: v.clone(),
            class: None,
            text_content: None,
            transforms: Vec::with_capacity(1),
            content: Vec::with_capacity(3),
        }
    }

    fn start_frame(&'s mut self, value: &AbsoulteRef, _group: &ir::GroupRef) -> Self::Group {
        let mut g = self.start_group(value);
        g.class = Some("group".to_owned());

        g
    }

    fn start_text(&'s mut self, value: &AbsoulteRef, text: &ir::FlatTextItem) -> Self::Group {
        let shape = &text.shape;

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

        let post_content = if self.render_text_element {
            // Scale is in pixel per em, but curve data is in font design units, so
            // we have to divide by units per em.
            let upem = shape.upem.0;
            let ppem = shape.ppem.0;
            let ascender = shape.ascender.0;

            // text_list.push(format!(
            //     r#"<h5:span textLength="{}" font-size="{}" class="tsel">{}</h5:span>"#,
            //     v,
            //     upem * ppem,
            //     xml::escape::escape_str_pcdata(&text.content.content),
            // ));

            let mut x = 0f32;
            for (offset, advance, _) in text.content.glyphs.iter() {
                x = x + offset.0 + advance.0;
            }

            // todo: investigate &nbsp;
            Some(format!(
                r#"<foreignObject x="0" y="-{}" width="{}" height="{}"><h5:div class="tsel" style="font-size: {}px;">{}</h5:div></foreignObject>"#,
                ascender,
                x,
                upem * ppem,
                upem * ppem,
                xml::escape::escape_str_pcdata(&text.content.content)
            ))
        } else {
            None
        };

        let mut g = self.start_group(value);

        g.class = Some(format!("typst-txt {}", fill));
        g.text_content = post_content;

        g
    }

    fn render_link(
        &'s mut self,
        abs_ref: AbsoulteRef,
        link: &crate::ir::LinkItem,
    ) -> Self::Resultant {
        let href_handler = if link.href.starts_with("@typst:") {
            let href = link.href.trim_start_matches("@typst:");
            format!(r##"xlink:href="#" onclick="{href}; return false""##)
        } else {
            format!(
                r##"target="_blank" xlink:href="{}""##,
                link.href.replace('&', "&amp;")
            )
        };

        format!(
            r#"<g data-tid="{}"><a {}><rect class="pseudo-link" width="{}" height="{}"></rect></a></g>"#,
            abs_ref.as_svg_id("l"),
            href_handler,
            link.size.x.0,
            link.size.y.0,
        )
    }

    fn render_path(&mut self, abs_ref: AbsoulteRef, path: &crate::ir::PathItem) -> Self::Resultant {
        format!(
            r#"<g data-tid="{}">{}</g>"#,
            abs_ref.as_svg_id("p"),
            render_path(path),
        )
    }

    fn render_image(
        &mut self,
        abs_ref: AbsoulteRef,
        image_item: &crate::ir::ImageItem,
    ) -> Self::Resultant {
        Self::render_image_inner(abs_ref, image_item)
    }
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
    let url = format!("data:image/{};base64,", image.format);

    let mut data = base64::engine::general_purpose::STANDARD.encode(&image.data);
    data.insert_str(0, &url);
    Some(data)
}

#[comemo::memoize]
fn render_path(path: &PathItem) -> String {
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
                p.push(format!(r#"stroke-width="{}" "#, width.0));
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
                    p.push(format!("{}", v.0));
                }
                p.push(r#"" "#.to_owned());
            }
            PathStyle::StrokeDashOffset(offset) => {
                p.push(format!(r#"stroke-dashoffset="{}" "#, offset.0));
            }
        }
    }
    p.push("/>".to_owned());
    p.join("")
}

/// Render a raster or SVG image into svg text.
// todo: error handling
pub fn render_image(image: &Image, size: Size) -> String {
    let image_url = rasterize_embedded_image_url(image).unwrap();

    // resize image to fit the view
    let size = size;
    let view_width = size.x.0;
    let view_height = size.y.0;

    let aspect = (image.width() as f32) / (image.height() as f32);

    let w = view_width.max(aspect * view_height);
    let h = w / aspect;
    format!(
        r#"<image x="0" y="0" width="{}" height="{}" xlink:href="{}" />"#,
        w, h, image_url
    )
}
