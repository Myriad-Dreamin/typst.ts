mod glyph;
mod text;

pub use glyph::SvgGlyphBuilder;
pub use reflexo::escape;
use reflexo::escape::AttributeEscapes;

use std::sync::Arc;

use base64::Engine;
use escape::PcDataEscapes;
use reflexo::hash::Fingerprint;
use reflexo::vector::{
    ir::{
        self, Abs, Axes, FontIndice, FontItem, GlyphRef, ImmutStr, PathStyle, Ratio, Scalar, Size,
        Transform,
    },
    vm::{GroupContext, IncrGroupContext, IncrRenderVm, RenderVm, TransformContext},
};
use reflexo_typst2vec::utils::ToCssExt;

pub trait BuildClipPath {
    fn build_clip_path(&mut self, path: &ir::PathItem) -> Fingerprint;
}

pub trait NotifyPaint {
    fn notify_paint(&mut self, url_ref: ImmutStr) -> (u8, Fingerprint, Option<Transform>);
}

pub trait DynExportFeature {
    fn should_render_text_element(&self) -> bool;

    fn use_stable_glyph_id(&self) -> bool;

    fn should_rasterize_text(&self) -> bool;

    fn should_attach_debug_info(&self) -> bool;

    fn should_aware_html_entity(&self) -> bool;
}

/// A generated text content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SvgText {
    /// Append a plain text.
    Plain(String),
    /// Append a SVG/XML text node.
    Content(Arc<SvgTextNode>),
}

impl SvgText {
    /// Recursively estimate the length of the text node for final string
    /// allocation.
    pub fn estimated_len(&self) -> usize {
        match self {
            Self::Plain(p) => p.len(),
            Self::Content(c) => c.estimated_len(),
        }
    }

    /// Recursively write the text content to the string.
    pub fn write_string_io(&self, string_io: &mut String) {
        match self {
            SvgText::Plain(c) => string_io.push_str(c),
            SvgText::Content(c) => c.write_string_io(string_io),
        }
    }

    pub fn join(data: Vec<SvgText>) -> String {
        generate_text(data)
    }
}

impl From<&str> for SvgText {
    fn from(s: &str) -> Self {
        SvgText::Plain(s.to_string())
    }
}

/// A generated text node in SVG/XML format.
/// The node is exactly the same as `<g>` tag.
/// It is formatted as `<g attr.keys()..="attr.values()..">content..</g>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SvgTextNode {
    pub attributes: Vec<(&'static str, String)>,
    pub content: Vec<SvgText>,
}

impl SvgTextNode {
    /// Recursively estimate the length of the text node for final string
    /// allocation.
    pub fn estimated_len(&self) -> usize {
        let content_estimated: usize = self.content.iter().map(SvgText::estimated_len).sum();
        let attr_estimated: usize = self
            .attributes
            .iter()
            .map(|attr| attr.0.len() + attr.1.len())
            .sum();

        "<g>".len()
            + (r#" ="""#.len() * self.attributes.len() + attr_estimated)
            + content_estimated
            + "</g>".len()
    }

    /// Recursively write the text content to the string.
    pub fn write_string_io(&self, string_io: &mut String) {
        string_io.push_str("<g");
        for (attr_name, attr_content) in &self.attributes {
            string_io.push(' ');
            string_io.push_str(attr_name);
            string_io.push('=');
            string_io.push('"');
            string_io.push_str(attr_content);
            string_io.push('"');
        }
        string_io.push('>');
        for c in &self.content {
            c.write_string_io(string_io)
        }
        string_io.push_str("</g>");
    }
}

#[derive(Clone, Copy)]
pub struct PaintObj {
    pub kind: u8,
    pub id: Fingerprint,
    pub transform: Option<Transform>,
}

/// A builder for [`SvgTextNode`].
/// It holds a reference to [`SvgRenderTask`] and state of the building process.
pub struct SvgTextBuilder {
    pub attributes: Vec<(&'static str, String)>,
    pub content: Vec<SvgText>,
    pub text_fill: Option<Arc<PaintObj>>,
    pub text_stroke: Option<Arc<PaintObj>>,
}

impl From<SvgTextBuilder> for Arc<SvgTextNode> {
    fn from(s: SvgTextBuilder) -> Self {
        Arc::new(SvgTextNode {
            attributes: s.attributes,
            content: s.content,
        })
    }
}

/// Internal methods for [`SvgTextBuilder`].
impl SvgTextBuilder {
    #[inline]
    pub fn render_text_semantics_inner(
        &mut self,
        shape: &ir::TextShape,
        content: &str,
        width: Scalar,
        ascender: Scalar,
        upem: Scalar,
    ) {
        // upem is the unit per em defined in the font.
        // ppem is calculated by the font size.
        // > ppem = text_size / upem
        let upem = upem.0;

        // because the text is already scaled by the font size,
        // we need to scale it back to the original size.
        // todo: infinite multiplication
        let ascender = ascender.0 * upem;
        let width = width.0 * upem / shape.size.0;

        let text_content = escape::escape_str::<PcDataEscapes>(content);

        // todo: investigate &nbsp;

        // we also apply some additional scaling.
        // so that the font-size doesn't hit the limit of the browser.
        // See <https://stackoverflow.com/questions/13416989/computed-font-size-is-bigger-than-defined-in-css-on-the-asus-nexus-7>
        self.content.push(SvgText::Plain(format!(
            concat!(
                // apply a negative scaleY to flip the text, since a glyph in font is
                // rendered upside down.
                r#"<g transform="scale(16,-16)">"#,
                r#"<foreignObject x="0" y="-{:.2}" width="{:.2}" height="{:.2}">"#,
                r#"<h5:div class="tsel" style="font-size: {}px">"#,
                "{}",
                r#"</h5:div></foreignObject></g>"#,
            ),
            ascender / 16.,
            width / 16.,
            upem / 16.,
            ((upem + 1e-3) / 16.) as u32,
            text_content
        )))
    }

    fn render_paint<C: NotifyPaint>(
        ctx: &mut C,
        color: ImmutStr,
        paint_id: &str,
    ) -> Option<SvgText> {
        let (kind, cano_ref, matrix) = ctx.notify_paint(color);

        Some(Self::transform_color(
            kind,
            paint_id,
            &cano_ref.as_svg_id("g"),
            matrix?,
        ))
    }

    fn transform_color(
        kind: u8,
        paint_id: &str,
        origin_id: &str,
        transform_matrix: Transform,
    ) -> SvgText {
        let tag = match kind {
            b'l' => "linearGradient",
            b'r' => "radialGradient",
            b'p' => "pattern",
            _ => unreachable!(),
        };

        let transform = match kind {
            b'p' => "patternTransform",
            _ => "gradientTransform",
        };

        SvgText::Plain(format!(
            r##"<{} id="{}" {}="{}" href="#{}" xlink:href="#{}"></{}>"##,
            tag,
            paint_id,
            transform,
            transform_matrix.to_css(),
            origin_id,
            origin_id,
            tag
        ))
    }

    fn render_paint_with_obj<C: NotifyPaint>(
        ctx: &mut C,
        color: ImmutStr,
        paint_id: &str,
    ) -> (u8, Option<Transform>, Option<SvgText>) {
        let (kind, cano_ref, matrix) = ctx.notify_paint(color);

        (
            kind,
            matrix,
            matrix.map(|matrix| {
                Self::transform_color(kind, paint_id, &cano_ref.as_svg_id("g"), matrix)
            }),
        )
    }

    pub fn render_glyph_slow(
        &mut self,
        pos: Axes<Scalar>,
        font: &FontItem,
        glyph: u32,
        fill: Option<Arc<PaintObj>>,
        stroke: Arc<PaintObj>,
    ) {
        let adjusted_x_offset = (pos.x.0 * 2.).round();
        let adjusted_y_offset = (pos.y.0 * 2.).round();

        // A stable glyph id can help incremental font transfer (IFT).
        // However, it is permitted unstable if you will not use IFT.
        let glyph_id = (GlyphRef {
            font_hash: font.hash,
            glyph_idx: glyph,
        })
        .as_svg_id("g");
        let mut do_trans = |obj: &PaintObj, pref: &'static str| -> String {
            let og = obj.id.as_svg_id(pref);
            let ng = format!("{og}-{adjusted_x_offset}-{adjusted_y_offset}").replace('.', "-");

            let new_color = Self::transform_color(
                obj.kind,
                &ng,
                &og,
                obj.transform
                    .unwrap_or_else(Transform::identity)
                    .post_concat(Transform::from_translate(
                        Scalar(-adjusted_x_offset / 2.),
                        Scalar(-adjusted_y_offset / 2.),
                    )),
            );

            self.content.push(new_color);

            ng
        };

        let fill_id = if let Some(fill) = fill {
            format!(r#" fill="url(#{})" "#, do_trans(&fill, "pf"))
        } else {
            String::default()
        };
        let stroke_id = format!(r#" stroke="url(#{})" "#, do_trans(&stroke, "ps"));

        self.content.push(SvgText::Plain(format!(
            // r##"<typst-glyph x="{}" href="#{}"/>"##,
            r##"<use x="{}" y="{}" href="#{}"{fill_id}{stroke_id}/>"##,
            adjusted_x_offset / 2.,
            adjusted_y_offset / 2.,
            glyph_id
        )));
    }
}

/// See [`TransformContext`].
impl<C: BuildClipPath> TransformContext<C> for SvgTextBuilder {
    fn transform_matrix(mut self, _ctx: &mut C, m: &ir::Transform) -> Self {
        self.attributes.push((
            "transform",
            format!(
                r#"matrix({},{},{},{},{},{})"#,
                m.sx.0, m.ky.0, m.kx.0, m.sy.0, m.tx.0, m.ty.0
            ),
        ));
        self
    }

    fn transform_translate(mut self, _ctx: &mut C, matrix: Axes<Abs>) -> Self {
        self.attributes.push((
            "transform",
            format!(r#"translate({:.3},{:.3})"#, matrix.x.0, matrix.y.0),
        ));
        self
    }

    fn transform_scale(mut self, _ctx: &mut C, x: Ratio, y: Ratio) -> Self {
        self.attributes
            .push(("transform", format!(r#"scale({},{})"#, x.0, y.0)));
        self
    }

    fn transform_rotate(mut self, _ctx: &mut C, matrix: Scalar) -> Self {
        self.attributes
            .push(("transform", format!(r#"rotate({})"#, matrix.0)));
        self
    }

    fn transform_skew(mut self, _ctx: &mut C, matrix: (Ratio, Ratio)) -> Self {
        self.attributes.push((
            "transform",
            format!(r#"skewX({}) skewY({})"#, matrix.0 .0, matrix.1 .0),
        ));
        self
    }

    fn transform_clip(mut self, ctx: &mut C, path: &ir::PathItem) -> Self {
        let clip_id = ctx.build_clip_path(path).as_svg_id("c");
        self.content.push(SvgText::Plain(format!(
            r##"<clipPath id="{}"><path d="{}"/></clipPath>"##,
            clip_id, path.d
        )));
        self.attributes
            .push(("clip-path", format!(r"url(#{clip_id})")));
        self
    }
}

/// See [`FlatGroupContext`].
impl<
        'm,
        C: NotifyPaint
            + RenderVm<'m, Resultant = Arc<SvgTextNode>>
            + FontIndice<'m>
            + DynExportFeature,
    > GroupContext<C> for SvgTextBuilder
{
    fn with_text_shape(
        &mut self,
        ctx: &mut C,
        upem: Scalar,
        shape: &ir::TextShape,
        context_key: &Fingerprint,
    ) {
        self.attributes.push(("class", "typst-text".to_owned()));

        if shape.styles.is_empty() {
            return;
        }

        let text_scale = upem.0 / shape.size.0;

        let (fill_id, stroke_id) =
            attach_path_styles(&shape.styles, Some(text_scale), &mut |x, y| {
                self.attributes.push((x, y));
            });

        let mut render_color_attr = |color: &Arc<str>, is_fill: bool| {
            let color = color.clone();
            if color.starts_with('@') {
                let paint_id = context_key.as_svg_id(if is_fill { "pf" } else { "ps" });
                let (kind, mat, content) = Self::render_paint_with_obj(ctx, color, &paint_id);
                (*(if is_fill {
                    &mut self.text_fill
                } else {
                    &mut self.text_stroke
                })) = Some(Arc::new(PaintObj {
                    kind,
                    id: *context_key,
                    transform: mat,
                }));
                if let Some(content) = content {
                    self.content.push(content);
                }
            } else {
                self.attributes
                    .push((if is_fill { "fill" } else { "stroke" }, color.to_string()));
            }
        };

        if let Some(color) = fill_id {
            render_color_attr(color, true);
        }
        if let Some(color) = stroke_id {
            render_color_attr(color, false);
        }
    }

    fn render_link(&mut self, _ctx: &mut C, link: &ir::LinkItem) {
        let href_handler = if link.href.starts_with("@typst:") {
            let href = link.href.trim_start_matches("@typst:");
            format!(r##"xlink:href="#" onclick="{href}; return false""##)
        } else {
            format!(
                r##"target="_blank" xlink:href="{}""##,
                link.href.replace('&', "&amp;").replace('"', "&quot;")
            )
        };

        self.content.push(SvgText::Plain(format!(
            r#"<a {}><rect class="pseudo-link" width="{}" height="{}"></rect></a>"#,
            href_handler, link.size.x.0, link.size.y.0,
        )))
    }

    fn render_path(&mut self, ctx: &mut C, path: &ir::PathItem, abs_ref: &Fingerprint) {
        let (fill_id, stroke_id, content) = render_path(path, abs_ref);

        let mut render_color_attr = |color: Arc<str>, is_fill: bool| {
            if color.starts_with('@') {
                let content = Self::render_paint(
                    ctx,
                    color,
                    &abs_ref.as_svg_id(if is_fill { "pf" } else { "ps" }),
                );
                if let Some(content) = content {
                    self.content.push(content);
                }
            }
        };

        if let Some(color) = fill_id {
            render_color_attr(color, true);
        }
        if let Some(color) = stroke_id {
            render_color_attr(color, false);
        }

        self.content.push(content);
    }

    fn render_image(&mut self, _ctx: &mut C, image_item: &ir::ImageItem) {
        self.content.push(render_image_item(image_item))
    }

    fn render_content_hint(&mut self, _ctx: &mut C, ch: char) {
        self.attributes
            .push(("class", "typst-content-hint".to_owned()));
        self.attributes
            .push(("data-hint", format!("{:x}", ch as u32)));
    }

    // HTML cannot be rendered in SVG sensibly.
    fn render_html(&mut self, _ctx: &mut C, _html: &ir::HtmlItem) {
        // self.content.push(SvgText::Plain(html.html.as_ref().into()))
    }

    #[inline]
    fn attach_debug_info(&mut self, ctx: &mut C, span_id: u64) {
        if ctx.should_attach_debug_info() {
            self.attributes.push(("data-span", format!("{span_id:x}")));
        }
    }
    fn render_item_at(&mut self, ctx: &mut C, pos: crate::ir::Point, item: &Fingerprint) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let sub_content = ctx.render_item(item);

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("transform", translate_attr),
                ("data-tid", item.as_svg_id("p")),
            ],
            content: vec![SvgText::Content(sub_content)],
        })));
    }

    fn render_glyph(&mut self, _ctx: &mut C, pos: Axes<Scalar>, font: &FontItem, glyph: u32) {
        let adjusted_x_offset = (pos.x.0 * 2.).round() / 2.;
        let adjusted_y_offset = (pos.y.0 * 2.).round() / 2.;

        // A stable glyph id can help incremental font transfer (IFT).
        // However, it is permitted unstable if you will not use IFT.
        let glyph_id = (GlyphRef {
            font_hash: font.hash,
            glyph_idx: glyph,
        })
        .as_svg_id("g");

        self.content.push(SvgText::Plain(format!(
            // r##"<typst-glyph x="{}" href="#{}"/>"##,
            r##"<use x="{adjusted_x_offset}" y="{adjusted_y_offset}" href="#{glyph_id}"/>"##
        )));
    }

    fn render_text_semantics(&mut self, ctx: &mut C, text: &ir::TextItem, width: Scalar) {
        if !ctx.should_render_text_element() {
            return;
        }

        let font = ctx.get_font(&text.shape.font).unwrap();

        self.render_text_semantics_inner(
            &text.shape,
            &text.content.content,
            width,
            font.ascender,
            font.units_per_em,
        )
    }

    fn with_frame(mut self, _ctx: &mut C, _group: &ir::GroupRef) -> Self {
        self.attributes.push(("class", "typst-group".to_owned()));
        self
    }

    fn with_text(mut self, ctx: &mut C, text: &ir::TextItem, fill_key: &Fingerprint) -> Self {
        let font = ctx.get_font(&text.shape.font).unwrap();
        let upem = font.units_per_em;

        self.with_text_shape(ctx, upem, &text.shape, fill_key);
        self
    }

    fn with_reuse(mut self, _ctx: &mut C, v: &Fingerprint) -> Self {
        self.attributes.push(("data-reuse-from", v.as_svg_id("g")));
        self
    }

    fn with_label(mut self, _ctx: &mut C, label: &str) -> Self {
        self.attributes.push(("data-typst-label", label.into()));
        self
    }
}

/// See [`FlatGroupContext`].
impl<'m, C: IncrRenderVm<'m, Resultant = Arc<SvgTextNode>, Group = SvgTextBuilder>>
    IncrGroupContext<C> for SvgTextBuilder
{
    fn render_diff_item_at(
        &mut self,
        ctx: &mut C,
        pos: crate::ir::Point,
        item: &Fingerprint,
        prev_item: &Fingerprint,
    ) {
        let content = if item == prev_item {
            // todo: update transform
            vec![]
        } else {
            let sub_content = ctx.render_diff_item(item, prev_item);
            vec![SvgText::Content(sub_content)]
        };

        let mut attributes = Vec::with_capacity(3);
        if pos != crate::ir::Point::default() {
            let transform_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);
            attributes.push(("transform", transform_attr));
        };
        attributes.push(("data-tid", item.as_svg_id("p")));
        attributes.push(("data-reuse-from", prev_item.as_svg_id("p")));

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes,
            content,
        })));
    }
}

fn attach_path_styles<'a>(
    styles: &'a [PathStyle],
    scale: Option<f32>,
    p: &mut impl FnMut(&'static str, String),
) -> (Option<&'a ImmutStr>, Option<&'a ImmutStr>) {
    let mut fill_color = None;
    let mut stroke_color = None;
    for style in styles.iter() {
        match style {
            PathStyle::Fill(color) => {
                fill_color = Some(color);
            }
            PathStyle::Stroke(color) => {
                stroke_color = Some(color);
            }
            PathStyle::StrokeWidth(width) => {
                p("stroke-width", (width.0 * scale.unwrap_or(1.)).to_string());
            }
            PathStyle::StrokeLineCap(cap) => {
                p("stroke-linecap", cap.to_string());
            }
            PathStyle::StrokeLineJoin(join) => {
                p("stroke-linejoin", join.to_string());
            }
            PathStyle::StrokeMitterLimit(limit) => {
                p(
                    "stroke-miterlimit",
                    (limit.0 * scale.unwrap_or(1.)).to_string(),
                );
            }
            PathStyle::StrokeDashArray(array) => {
                p(
                    "stroke-dasharray",
                    array
                        .iter()
                        .map(|e| (e.0 * scale.unwrap_or(1.)).to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
            PathStyle::StrokeDashOffset(offset) => {
                p(
                    "stroke-dashoffset",
                    (offset.0 * scale.unwrap_or(1.)).to_string(),
                );
            }
            PathStyle::FillRule(rule) => {
                p("fill-rule", rule.to_string());
            }
        }
    }

    (fill_color, stroke_color)
}

/// Render a [`ir::PathItem`] into svg text.
#[comemo::memoize]
fn render_path(
    path: &ir::PathItem,
    abs_ref: &Fingerprint,
) -> (Option<ImmutStr>, Option<ImmutStr>, SvgText) {
    let mut p = vec![r#"<path class="typst-shape" "#.to_owned()];
    p.push(format!(r#"d="{}" "#, path.d));

    let (fill_color, stroke_color) = attach_path_styles(&path.styles, None, &mut |x, y| {
        p.push(format!(r#"{x}="{y}" "#))
    });

    let contextual_id = |id: &'static str| abs_ref.as_svg_id(id);
    if let Some(fill_color) = fill_color {
        if fill_color.starts_with('@') {
            p.push(format!(r#"fill="url(#{})" "#, contextual_id("pf")));
        } else {
            p.push(format!(r#"fill="{fill_color}" "#));
        }
    } else {
        p.push(r#"fill="none" "#.to_string());
    }
    if let Some(stroke_color) = stroke_color {
        if stroke_color.starts_with('@') {
            p.push(format!(r#"stroke="url(#{})" "#, contextual_id("ps")));
        } else {
            p.push(format!(r#"stroke="{stroke_color}" "#));
        }
    }
    p.push("/>".to_owned());
    (
        fill_color.cloned(),
        stroke_color.cloned(),
        SvgText::Plain(p.join("")),
    )
}

/// Render a [`ir::ImageItem`] into svg text.
#[comemo::memoize]
fn render_image_item(img: &ir::ImageItem) -> SvgText {
    SvgText::Plain(render_image(&img.image, img.size, true, ""))
}

/// Render a raster or SVG image into svg text.
/// is_image_elem: whether the image is an `<image>` element (instead of an
/// image glyph).
/// style: additional style attribute.
// todo: error handling
pub fn render_image(image: &ir::Image, size: Size, is_image_elem: bool, style: &str) -> String {
    let image_url = embed_as_image_url(image).unwrap();

    let styles = image.attrs.iter().map(|attr| match attr {
        ir::ImageAttr::Alt(alt) => {
            format!(r#" alt="{}""#, escape::escape_str::<AttributeEscapes>(alt))
        }
        ir::ImageAttr::ImageRendering(rendering) => format!(r#" image-rendering="{rendering}""#),
    });
    let styles = styles.collect::<Vec<_>>().join(" ");

    let w = size.x.0;
    let h = size.y.0;

    let cls = if is_image_elem {
        r#" class="typst-image""#
    } else {
        ""
    };
    format!(
        r#"<image{cls} width="{w}" height="{h}" xlink:href="{image_url}" preserveAspectRatio="none"{style}{styles}/>"#,
    )
}

fn embed_as_image_url(image: &ir::Image) -> Option<String> {
    let url = format!("data:image/{};base64,", image.format);

    let mut data = base64::engine::general_purpose::STANDARD.encode(&image.data);
    data.insert_str(0, &url);
    Some(data)
}

/// Concatenate a list of [`SvgText`] into a single string.
pub fn generate_text(text_list: Vec<SvgText>) -> String {
    let mut string_io = String::new();
    string_io.reserve(text_list.iter().map(SvgText::estimated_len).sum());
    for s in text_list {
        s.write_string_io(&mut string_io);
    }
    string_io
}
