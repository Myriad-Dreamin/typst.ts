mod escape;
mod glyph;
mod text;

pub use glyph::SvgGlyphBuilder;

use std::sync::Arc;

use base64::Engine;
use escape::{PcDataEscapes, TextContentDataEscapes};
use typst_ts_core::{
    hash::Fingerprint,
    vector::{
        ir::{
            self, Abs, Axes, BuildGlyph, FontIndice, GlyphHashStablizer, GlyphIndice, GlyphRef,
            ImmutStr, PathStyle, Ratio, Scalar, Size, Transform,
        },
        vm::{
            GroupContext, IncrGroupContext, IncrRenderVm, RenderState, RenderVm, TransformContext,
        },
    },
};

use crate::{frontend::HasStatefulFill, utils::ToCssExt};

pub trait BuildClipPath {
    fn build_clip_path(&mut self, path: &ir::PathItem) -> Fingerprint;
}

pub trait BuildFillStyleClass {
    fn build_fill_style_class(&mut self, fill: ImmutStr) -> String;
}

pub trait NotifyPaint {
    fn notify_paint(&mut self, url_ref: ImmutStr) -> (u8, Fingerprint, Option<bool>);
}

pub trait DynExportFeature {
    fn enable_inlined_svg(&self) -> bool;

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

/// A builder for [`SvgTextNode`].
/// It holds a reference to [`SvgRenderTask`] and state of the building process.
pub struct SvgTextBuilder {
    pub attributes: Vec<(&'static str, String)>,
    pub content: Vec<SvgText>,
    pub text_fill: Option<Fingerprint>,
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
    pub fn render_glyph_inner<C: DynExportFeature + GlyphHashStablizer>(
        &mut self,
        ctx: &mut C,
        pos: Scalar,
        glyph: &GlyphRef,
    ) {
        let adjusted_offset = (pos.0 * 2.).round() / 2.;

        // A stable glyph id can help incremental font transfer (IFT).
        // However, it is permitted unstable if you will not use IFT.
        let glyph_id = if ctx.use_stable_glyph_id() {
            ctx.stablize_hash(glyph).as_svg_id("g")
        } else {
            glyph.as_unstable_svg_id("g")
        };

        self.content.push(SvgText::Plain(format!(
            // r##"<typst-glyph x="{}" href="#{}"/>"##,
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
        ascender: Scalar,
        upem: Scalar,
        aware_html_entity: bool,
    ) {
        // upem is the unit per em defined in the font.
        // ppem is calcuated by the font size.
        // > ppem = text_size / upem
        let upem = upem.0;

        // because the text is already scaled by the font size,
        // we need to scale it back to the original size.
        // todo: infinite multiplication
        let ascender = ascender.0 * upem;
        let width = width.0 * upem / shape.size.0;

        let text_content = if aware_html_entity {
            escape::escape_str::<TextContentDataEscapes>(content)
        } else {
            escape::escape_str::<PcDataEscapes>(content)
        };

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
            .push(("clip-path", format!(r"url(#{})", clip_id)));
        self
    }
}

/// See [`FlatGroupContext`].
impl<
        'm,
        C: BuildGlyph
            + GlyphHashStablizer
            + GlyphIndice<'m>
            + NotifyPaint
            + RenderVm<'m, Resultant = Arc<SvgTextNode>>
            + FontIndice<'m>
            + BuildFillStyleClass
            + DynExportFeature,
    > GroupContext<C> for SvgTextBuilder
{
    fn with_text_shape(
        &mut self,
        ctx: &mut C,
        upem: Scalar,
        shape: &ir::TextShape,
        fill_key: &Fingerprint,
        state: RenderState,
    ) {
        let color = &shape.fill;
        let fill_id = if color.starts_with('@') {
            let (kind, cano_ref, relative_to_self) = ctx.notify_paint(shape.fill.clone());
            let text_scale = upem.0 / shape.size.0;
            let text_scale = Transform::from_scale(Scalar(text_scale), Scalar(-text_scale));

            let relative_to_self = relative_to_self.unwrap_or(false);
            let is_gradient = color.starts_with("@g");
            let mat = if is_gradient {
                if relative_to_self {
                    text_scale
                } else {
                    state.body_inv_transform().post_concat(text_scale)
                }
            } else if relative_to_self {
                text_scale
            } else {
                // println!("state: {:?}", state.inv_transform());
                state.inv_transform().post_concat(text_scale)
            };
            let mat = mat.to_css();

            // abs_ref
            let paint_id = fill_key.as_svg_id("tf");
            self.text_fill = Some(*fill_key);

            let decl = transform_paint_fill(kind, cano_ref, &paint_id, &mat);
            self.content.push(decl);
            "".to_owned()
        } else {
            ctx.build_fill_style_class(color.clone())
        };

        self.attributes
            .push(("class", format!("typst-text {}", fill_id)));
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

    fn render_path(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        path: &ir::PathItem,
        abs_ref: &Fingerprint,
    ) {
        for s in &path.styles {
            match s {
                PathStyle::Fill(color) | PathStyle::Stroke(color) => {
                    if color.starts_with('@') {
                        // todo: whether we need to distinguish fill and stroke here?
                        let is_fill = matches!(s, PathStyle::Fill(..));
                        let is_gradient = color.starts_with("@g");

                        // todo
                        let (kind, cano_ref, relative_to_self) = ctx.notify_paint(color.clone());

                        let relative_to_self = relative_to_self.unwrap_or(true);

                        let transform_matrix = if is_gradient {
                            if relative_to_self {
                                let self_bbox = path.size.unwrap();
                                Transform::from_scale(self_bbox.x, self_bbox.y)
                            } else {
                                // println!("state: {:?}", state.inv_transform());
                                state.body_inv_transform()
                            }
                        } else if relative_to_self {
                            Transform::identity()
                        } else {
                            // println!("state: {:?}", state.inv_transform());
                            state.inv_transform()
                        };
                        let transform_matrix = transform_matrix.to_css();

                        let paint_id =
                            state
                                .at(abs_ref)
                                .as_svg_id(if is_fill { "pf" } else { "ps" });

                        let decl =
                            transform_paint_fill(kind, cano_ref, &paint_id, &transform_matrix);

                        self.content.push(decl);
                    }
                }
                _ => {}
            }
        }
        self.content.push(render_path(path, state, abs_ref))
    }

    fn render_image(&mut self, ctx: &mut C, image_item: &ir::ImageItem) {
        self.content
            .push(render_image_item(image_item, ctx.enable_inlined_svg()))
    }

    fn render_content_hint(&mut self, _ctx: &mut C, ch: char) {
        self.attributes
            .push(("class", "typst-content-hint".to_owned()));
        self.attributes
            .push(("data-hint", format!("{:x}", ch as u32)));
    }

    #[inline]
    fn attach_debug_info(&mut self, ctx: &mut C, span_id: u64) {
        if ctx.should_attach_debug_info() {
            self.attributes
                .push(("data-span", format!("{:x}", span_id)));
        }
    }
    fn render_item_ref_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        pos: crate::ir::Point,
        item: &Fingerprint,
    ) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let sub_content = ctx.render_flat_item(state, item);

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("transform", translate_attr),
                ("data-tid", item.as_svg_id("p")),
            ],
            content: vec![SvgText::Content(sub_content)],
        })));
    }

    fn render_glyph_ref(&mut self, ctx: &mut C, pos: Scalar, glyph: &GlyphRef) {
        self.render_glyph_inner(ctx, pos, glyph)
    }

    fn render_flat_text_semantics(&mut self, ctx: &mut C, text: &ir::TextItem, width: Scalar) {
        if !ctx.should_render_text_element() {
            return;
        }

        let font = ctx.get_font(&text.font).unwrap();

        self.render_text_semantics_inner(
            &text.shape,
            &text.content.content,
            width,
            font.ascender,
            font.unit_per_em,
            ctx.should_aware_html_entity(),
        )
    }

    fn with_frame(mut self, _ctx: &mut C, _group: &ir::GroupRef) -> Self {
        self.attributes.push(("class", "typst-group".to_owned()));
        self
    }

    fn with_text(
        mut self,
        ctx: &mut C,
        text: &ir::TextItem,
        fill_key: &Fingerprint,
        state: RenderState,
    ) -> Self {
        let font = ctx.get_font(&text.font).unwrap();
        let upem = font.unit_per_em;

        self.with_text_shape(ctx, upem, &text.shape, &state.at(fill_key), state);
        self
    }

    fn with_reuse(mut self, _ctx: &mut C, v: &Fingerprint) -> Self {
        self.attributes.push(("data-reuse-from", v.as_svg_id("g")));
        self
    }
}

/// See [`FlatGroupContext`].
impl<
        'm,
        C: IncrRenderVm<'m, Resultant = Arc<SvgTextNode>, Group = SvgTextBuilder> + HasStatefulFill,
    > IncrGroupContext<C> for SvgTextBuilder
{
    fn render_diff_item_ref_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        pos: crate::ir::Point,
        item: &Fingerprint,
        prev_item: &Fingerprint,
    ) {
        let has_stateful_fill = ctx.has_stateful_fill(item);
        let content = if item == prev_item && !has_stateful_fill {
            // todo: update transform
            vec![]
        } else {
            let sub_content = ctx.render_diff_item(state, item, prev_item);
            vec![SvgText::Content(sub_content)]
        };

        let mut attributes = Vec::with_capacity(3);
        if pos != crate::ir::Point::default() {
            let transform_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);
            attributes.push(("transform", transform_attr));
        };
        attributes.push(("data-tid", item.as_svg_id("p")));
        attributes.push(("data-reuse-from", prev_item.as_svg_id("p")));
        if has_stateful_fill {
            attributes.push(("data-bad-equality", "1".to_owned()));
        }

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes,
            content,
        })));
    }
}

/// Render a [`ir::PathItem`] into svg text.
#[comemo::memoize]
fn render_path(path: &ir::PathItem, state: RenderState, abs_ref: &Fingerprint) -> SvgText {
    let mut p = vec![r#"<path class="typst-shape" "#.to_owned()];
    p.push(format!(r#"d="{}" "#, path.d));
    #[allow(unused_assignments)]
    let mut ft = String::new();
    let mut fill_color = "none";
    for style in &path.styles {
        match style {
            PathStyle::Fill(color) => {
                fill_color = if color.starts_with('@') {
                    ft = format!(r#"url(#{})"#, state.at(abs_ref).as_svg_id("pf"));
                    &ft
                } else {
                    color
                };
            }
            PathStyle::Stroke(color) => {
                // compress the stroke color
                p.push(if color.starts_with('@') {
                    let ps = state.at(abs_ref).as_svg_id("ps");
                    format!(r##"stroke="url(#{})" "##, &ps)
                } else {
                    format!(r##"stroke="{}" "##, color)
                });
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
                p.push(r#"stroke-dasharray=""#.to_owned());
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
    SvgText::Plain(p.join(""))
}

/// Render a [`ir::ImageItem`] into svg text.
#[comemo::memoize]
fn render_image_item(img: &ir::ImageItem, enable_inlined: bool) -> SvgText {
    if enable_inlined {
        match &img.image.alt {
            Some(t) if t.as_ref() == "!typst-inlined-svg" => {
                return SvgText::Plain(String::from_utf8(img.image.data.clone()).unwrap())
            }
            _ => {}
        }
    }

    SvgText::Plain(render_image(&img.image, img.size, true, ""))
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

fn transform_paint_fill(
    kind: u8,
    f: Fingerprint,
    paint_id: &str,
    transform_matrix: &str,
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
        transform_matrix,
        f.as_svg_id("g"),
        f.as_svg_id("g"),
        tag
    ))
}
