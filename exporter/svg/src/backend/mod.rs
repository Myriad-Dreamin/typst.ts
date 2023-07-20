use std::sync::Arc;

use base64::Engine;

use typst_ts_core::{
    font::GlyphProvider,
    hash::Fingerprint,
    vector::{
        flat_ir,
        flat_vm::{FlatGroupContext, FlatIncrGroupContext, FlatIncrRenderVm, FlatRenderVm},
        ir::{self, Abs, AbsoluteRef, Axes, ImmutStr, PathStyle, Ratio, Scalar, Size},
        vm::{GroupContext, RenderVm, TransformContext},
        GlyphLowerBuilder,
    },
};

pub(crate) mod debug_info;
pub use debug_info::generate_src_mapping;

mod escape;
use escape::TextContentDataEscapes;

use crate::utils::ToCssExt;

pub trait BuildGlyph {
    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> AbsoluteRef;
}

pub trait BuildClipPath {
    fn build_clip_path(&mut self, path: &ir::PathItem) -> Fingerprint;
}

pub trait BuildFillStyleClass {
    fn build_fill_style_class(&mut self, fill: ImmutStr) -> String;
}

pub trait DynExportFeature {
    fn should_render_text_element(&self) -> bool;

    fn use_stable_glyph_id(&self) -> bool;

    fn should_attach_debug_info(&self) -> bool;
}

/// A generated text content.
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
}

impl From<&str> for SvgText {
    fn from(s: &str) -> Self {
        SvgText::Plain(s.to_string())
    }
}

/// A generated text node in SVG/XML format.
/// The node is exactly the same as `<g>` tag.
/// It is formatted as `<g attr.keys()..="attr.values()..">content..</g>`.
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

pub struct SvgGlyphBuilder {
    pub glyph_provider: GlyphProvider,
}

impl SvgGlyphBuilder {
    pub fn render_glyph(&mut self, glyph_id: &str, glyph_item: &ir::GlyphItem) -> Option<String> {
        let gp = &self.glyph_provider;
        Self::render_glyph_inner(gp, glyph_id, glyph_item)
    }

    #[comemo::memoize]
    pub fn render_glyph_pure(glyph_id: &str, glyph_item: ir::GlyphItem) -> Option<String> {
        Self::render_glyph_pure_inner(glyph_id, &glyph_item)
    }

    #[comemo::memoize]
    fn render_glyph_inner(
        gp: &GlyphProvider,
        glyph_id: &str,
        glyph_item: &ir::GlyphItem,
    ) -> Option<String> {
        if matches!(glyph_item, ir::GlyphItem::Raw(..)) {
            return Self::render_glyph_pure_inner(
                glyph_id,
                &GlyphLowerBuilder::new(gp).lower_glyph(glyph_item)?,
            );
        }

        Self::render_glyph_pure_inner(glyph_id, glyph_item)
    }

    fn render_glyph_pure_inner(glyph_id: &str, glyph_item: &ir::GlyphItem) -> Option<String> {
        match glyph_item {
            ir::GlyphItem::Image(image_glyph) => Self::render_image_glyph(glyph_id, image_glyph),
            ir::GlyphItem::Outline(outline_glyph) => {
                Self::render_outline_glyph(glyph_id, outline_glyph)
            }
            ir::GlyphItem::Raw(..) => unreachable!(),
        }
    }

    /// Render an image glyph into the svg text.
    fn render_image_glyph(glyph_id: &str, ig: &ir::ImageGlyphItem) -> Option<String> {
        let transform_style = format!(r#" style="transform: {}""#, ig.ts.to_css());

        let img = render_image(&ig.image.image, ig.image.size, false, &transform_style);

        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="image_glyph">{}</symbol>"#,
            glyph_id, img
        );
        Some(symbol_def)
    }

    /// Render an outline glyph into svg text. This is the "normal" case.
    fn render_outline_glyph(
        glyph_id: &str,
        outline_glyph: &ir::OutlineGlyphItem,
    ) -> Option<String> {
        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="outline_glyph"><path d="{}"/></symbol>"#,
            glyph_id, outline_glyph.d
        );
        Some(symbol_def)
    }
}

/// A builder for [`SvgTextNode`].
/// It holds a reference to [`SvgRenderTask`] and state of the building process.
pub struct SvgTextBuilder {
    pub attributes: Vec<(&'static str, String)>,
    pub content: Vec<SvgText>,
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
    pub fn render_glyph_inner<C: DynExportFeature>(
        &mut self,
        ctx: &mut C,
        pos: Scalar,
        glyph: &AbsoluteRef,
    ) {
        let adjusted_offset = (pos.0 * 2.).round() / 2.;

        // A stable glyph id can help incremental font transfer (IFT).
        // However, it is permitted unstable if you will not use IFT.
        let glyph_id = if ctx.use_stable_glyph_id() {
            glyph.as_svg_id("g")
        } else {
            glyph.as_unstable_svg_id("g")
        };

        self.content.push(SvgText::Plain(format!(
            r##"<use x="{}" href="#{}"/>"##,
            adjusted_offset, glyph_id
        )));
    }

    #[inline]
    pub fn render_text_semantics_inner(
        &mut self,
        shape: &ir::TextShape,
        content: &str,
        width: Scalar,
    ) {
        // upem is the unit per em defined in the font.
        // ppem is calcuated by the font size.
        // > ppem = text_size / upem
        let upem = shape.upem.0;
        let ppem = shape.ppem.0;

        // because the text is already scaled by the font size,
        // we need to scale it back to the original size.
        let ascender = shape.ascender.0 / ppem;
        let width = width.0 / ppem;

        let text_content = escape::escape_str::<TextContentDataEscapes>(content);

        // todo: investigate &nbsp;
        self.content.push(SvgText::Plain(format!(
            concat!(
                // apply a negative scaleY to flip the text, since a glyph in font is
                // rendered upside down.
                r#"<g transform="scale(1,-1)">"#,
                r#"<foreignObject x="0" y="-{}" width="{}" height="{}">"#,
                r#"<h5:div class="tsel" style="font-size: {}px">"#,
                "{}",
                r#"</h5:div></foreignObject></g>"#,
            ),
            ascender, width, upem, upem, text_content
        )))
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
        let clip_id = ctx.build_clip_path(path);

        self.attributes
            .push(("clip-path", format!(r"url(#{})", clip_id.as_svg_id("c"))));
        self
    }
}

/// See [`GroupContext`].
impl<
        C: RenderVm<Resultant = Arc<SvgTextNode>>
            + BuildGlyph
            + BuildFillStyleClass
            + DynExportFeature,
    > GroupContext<C> for SvgTextBuilder
{
    fn with_text_shape(&mut self, ctx: &mut C, shape: &ir::TextShape) {
        // shorten black fill
        let fill_id = ctx.build_fill_style_class(shape.fill.clone());

        self.attributes
            .push(("class", format!("typst-text {}", fill_id)));
    }

    fn render_item_at(&mut self, ctx: &mut C, pos: ir::Point, item: &ir::SvgItem) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let sub_content = ctx.render_item(item);

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![("transform", translate_attr)],
            content: vec![SvgText::Content(sub_content)],
        })));
    }

    /// Assuming the glyphs has already been in the defs, render it by
    /// reference.
    #[inline]
    fn render_glyph(&mut self, ctx: &mut C, pos: Scalar, glyph: &ir::GlyphItem) {
        let glyph_ref = ctx.build_glyph(glyph);

        self.render_glyph_inner(ctx, pos, &glyph_ref)
    }

    fn render_link(&mut self, _ctx: &mut C, link: &ir::LinkItem) {
        let href_handler = if link.href.starts_with("@typst:") {
            let href = link.href.trim_start_matches("@typst:");
            format!(r##"xlink:href="#" onclick="{href}; return false""##)
        } else {
            format!(
                r##"target="_blank" xlink:href="{}""##,
                link.href.replace('&', "&amp;")
            )
        };

        self.content.push(SvgText::Plain(format!(
            r#"<a {}><rect class="pseudo-link" width="{}" height="{}"></rect></a>"#,
            href_handler, link.size.x.0, link.size.y.0,
        )))
    }

    fn render_path(&mut self, _ctx: &mut C, path: &ir::PathItem) {
        self.content.push(SvgText::Plain(render_path(path)))
    }

    fn render_image(&mut self, _ctx: &mut C, image_item: &ir::ImageItem) {
        self.content.push(SvgText::Plain(render_image(
            &image_item.image,
            image_item.size,
            true,
            "",
        )))
    }

    fn render_semantic_text(&mut self, ctx: &mut C, text: &ir::TextItem, width: Scalar) {
        if !ctx.should_render_text_element() {
            return;
        }

        self.render_text_semantics_inner(&text.shape, &text.content.content, width)
    }

    #[inline]
    fn attach_debug_info(&mut self, ctx: &mut C, span_id: u64) {
        if ctx.should_attach_debug_info() {
            self.attributes
                .push(("data-span", format!("{:x}", span_id)));
        }
    }
}

/// See [`FlatGroupContext`].
impl<
        'm,
        C: RenderVm<Resultant = Arc<SvgTextNode>>
            + FlatRenderVm<'m, Resultant = Arc<SvgTextNode>>
            + BuildGlyph
            + BuildFillStyleClass
            + DynExportFeature,
    > FlatGroupContext<C> for SvgTextBuilder
{
    fn render_item_ref_at(&mut self, ctx: &mut C, pos: crate::ir::Point, item: &AbsoluteRef) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let sub_content = ctx.render_flat_item(item);

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("transform", translate_attr),
                ("data-tid", item.as_svg_id("p")),
            ],
            content: vec![SvgText::Content(sub_content)],
        })));
    }

    fn render_glyph_ref(&mut self, ctx: &mut C, pos: Scalar, glyph: &AbsoluteRef) {
        self.render_glyph_inner(ctx, pos, glyph)
    }

    fn render_flat_text_semantics(
        &mut self,
        ctx: &mut C,
        text: &flat_ir::FlatTextItem,
        width: Scalar,
    ) {
        if !ctx.should_render_text_element() {
            return;
        }

        self.render_text_semantics_inner(&text.shape, &text.content.content, width)
    }

    fn with_frame(mut self, _ctx: &mut C, _group: &flat_ir::GroupRef) -> Self {
        self.attributes.push(("class", "typst-group".to_owned()));
        self
    }

    fn with_text(mut self, ctx: &mut C, text: &flat_ir::FlatTextItem) -> Self {
        self.with_text_shape(ctx, &text.shape);
        self
    }

    fn with_reuse(mut self, _ctx: &mut C, v: &AbsoluteRef) -> Self {
        self.attributes.push(("data-reuse-from", v.as_svg_id("g")));
        self
    }
}

/// See [`FlatGroupContext`].
impl<'m, C: FlatIncrRenderVm<'m, Resultant = Arc<SvgTextNode>, Group = SvgTextBuilder>>
    FlatIncrGroupContext<C> for SvgTextBuilder
{
    fn render_diff_item_ref_at(
        &mut self,
        ctx: &mut C,
        pos: crate::ir::Point,
        item: &AbsoluteRef,
        prev_item: &AbsoluteRef,
    ) {
        let content = if item == prev_item {
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

/// Render a [`PathItem`] into svg text.
#[comemo::memoize]
fn render_path(path: &ir::PathItem) -> String {
    let mut p = vec![r#"<path class="typst-shape" "#.to_owned()];
    p.push(format!(r#"d="{}" "#, path.d));
    let mut fill_color = "none";
    for style in &path.styles {
        match style {
            PathStyle::Fill(color) => {
                fill_color = color;
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
    p.push(format!(r#"fill="{}" "#, fill_color));
    p.push("/>".to_owned());
    p.join("")
}

/// Render a raster or SVG image into svg text.
/// is_image_elem: whether the image is an `<image>` element (instead of an
/// image glyph).
/// style: additional style attribute.
// todo: error handling
pub fn render_image(image: &ir::Image, size: Size, is_image_elem: bool, style: &str) -> String {
    let image_url = embed_as_image_url(image).unwrap();

    let w = size.x.0;
    let h = size.y.0;

    let cls = if is_image_elem {
        r#" class="typst-image""#
    } else {
        ""
    };
    format!(
        r#"<image{cls} width="{w}" height="{h}" xlink:href="{image_url}" preserveAspectRatio="none"{style}/>"#,
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
