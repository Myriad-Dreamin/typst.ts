use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use reflexo::vector::ir::Transform;
use typst_ts_core::{
    hash::{Fingerprint, FingerprintBuilder},
    vector::{
        ir::{
            self, FlatGlyphItem, FontIndice, FontRef, GroupRef, ImmutStr, Module, PathItem, Scalar,
            TextItem, VecItem,
        },
        vm::{GroupContext, IncrRenderVm, RenderVm},
    },
};

use crate::{
    backend::{BuildClipPath, DynExportFeature, NotifyPaint, SvgText, SvgTextBuilder, SvgTextNode},
    ExportFeature,
};

// unused
/// Global style namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
// #[repr(u32)]
pub enum StyleNs {
    // style that contains a single css rule: `fill: #color`.
    // Fill,
    // style that contains a single css rule: `stroke: #color`.
    // Stroke,
}

/// Maps the style name to the style definition.
/// See [`StyleNs`].
pub(crate) type StyleDefMap = HashMap<(StyleNs, ImmutStr), String>;
/// Maps paint fill id to the paint fill's data.
pub(crate) type PaintFillMap = HashSet<Fingerprint>;

/// The task context for rendering vector items
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of Vector task.
pub struct RenderContext<'m, 't, Feat: ExportFeature> {
    pub module: &'m Module,

    /// A fingerprint builder for generating unique id.
    pub(crate) fingerprint_builder: &'t mut FingerprintBuilder,

    /// Stores the style definitions used in the document.
    pub(crate) _style_defs: &'t mut StyleDefMap,
    /// Stores the graidents used in the document.
    pub(crate) gradients: &'t mut PaintFillMap,
    /// Stores the patterns used in the document.
    pub(crate) patterns: &'t mut PaintFillMap,

    /// See [`ExportFeature`].
    pub should_render_text_element: bool,
    /// See [`ExportFeature`].
    pub should_attach_debug_info: bool,
    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,
    /// See [`ExportFeature`].
    pub should_rasterize_text: bool,

    pub _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<'m, 't, Feat: ExportFeature> DynExportFeature for RenderContext<'m, 't, Feat> {
    #[inline]
    fn should_render_text_element(&self) -> bool {
        Feat::SHOULD_RENDER_TEXT_ELEMENT && self.should_render_text_element
    }

    #[inline]
    fn use_stable_glyph_id(&self) -> bool {
        Feat::USE_STABLE_GLYPH_ID && self.use_stable_glyph_id
    }

    #[inline]
    fn should_rasterize_text(&self) -> bool {
        Feat::SHOULD_RASTERIZE_TEXT && self.should_rasterize_text
    }

    #[inline]
    fn should_attach_debug_info(&self) -> bool {
        Feat::SHOULD_ATTACH_DEBUG_INFO && self.should_attach_debug_info
    }

    #[inline]
    fn should_aware_html_entity(&self) -> bool {
        Feat::AWARE_HTML_ENTITY
    }
}

impl<'m, 't, Feat: ExportFeature> FontIndice<'m> for RenderContext<'m, 't, Feat> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize).map(|e| {
            // canary check
            if e.hash != value.hash {
                panic!("Invalid font reference: {:?}", value);
            }

            e
        })
    }
}

impl<'m, 't, Feat: ExportFeature> BuildClipPath for RenderContext<'m, 't, Feat> {
    fn build_clip_path(&mut self, path: &PathItem) -> Fingerprint {
        self.fingerprint_builder.resolve(path)
    }
}

impl<'m, 't, Feat: ExportFeature> NotifyPaint for RenderContext<'m, 't, Feat> {
    fn notify_paint(&mut self, url_ref: ImmutStr) -> (u8, Fingerprint, Option<Transform>) {
        if url_ref.starts_with("@g") {
            let id = url_ref.trim_start_matches("@g");
            let mut id = Fingerprint::try_from_str(id).unwrap();

            let transform = match self.get_item(&id) {
                Some(VecItem::ColorTransform(g)) => {
                    id = g.item;
                    Some(g.transform)
                }
                _ => None,
            };

            let kind = match self.get_item(&id) {
                Some(VecItem::Gradient(g)) => &g.kind,
                _ => {
                    // #[cfg(debug_assertions)]
                    panic!("Invalid gradient reference: {}", id.as_svg_id("g"));
                }
            };

            let kind = match kind {
                ir::GradientKind::Linear(..) => b'l',
                ir::GradientKind::Radial(..) => b'r',
                ir::GradientKind::Conic(..) => b'p',
            };

            self.gradients.insert(id);
            (kind, id, transform)
        } else if url_ref.starts_with("@p") {
            let id = url_ref.trim_start_matches("@p");
            let mut id = Fingerprint::try_from_str(id).unwrap();

            let transform = match self.get_item(&id) {
                Some(VecItem::ColorTransform(g)) => {
                    id = g.item;
                    Some(g.transform)
                }
                _ => None,
            };

            let kind = b'p';

            self.patterns.insert(id);
            (kind, id, transform)
        } else {
            panic!("Invalid url reference: {}", url_ref);
        }
    }
}

/// Example of how to implement a FlatRenderVm.
impl<'m, 't, Feat: ExportFeature> RenderVm<'m> for RenderContext<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = Arc<SvgTextNode>;
    type Group = SvgTextBuilder;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m VecItem> {
        self.module.get_item(value)
    }

    fn start_group(&mut self, v: &Fingerprint) -> Self::Group {
        Self::Group {
            attributes: vec![("data-tid", v.as_svg_id("g"))],
            text_fill: None,
            text_stroke: None,
            content: Vec::with_capacity(1),
        }
    }

    fn start_frame(&mut self, value: &Fingerprint, _group: &GroupRef) -> Self::Group {
        let mut g = self.start_group(value);
        g.attributes.push(("class", "typst-group".to_owned()));
        g
    }

    fn start_text(&mut self, value: &Fingerprint, text: &TextItem) -> Self::Group {
        let mut g = self.start_group(value);

        let font = self.get_font(&text.shape.font).unwrap();
        let upem = font.units_per_em;

        g.with_text_shape(self, upem, &text.shape, value);
        g
    }

    /// Render a text into the underlying context.
    fn render_text(
        &mut self,
        group_ctx: Self::Group,
        abs_ref: &Fingerprint,
        text: &TextItem,
    ) -> Self::Group {
        if self.should_rasterize_text() {
            self.rasterize_and_put_text(group_ctx, abs_ref, text)
        } else {
            self.render_text_inplace(group_ctx, text)
        }
    }
}

impl<'m, 't, Feat: ExportFeature> IncrRenderVm<'m> for RenderContext<'m, 't, Feat> {}

#[cfg(not(feature = "aggresive-browser-rasterization"))]
impl<'m, 't, Feat: ExportFeature> RenderContext<'m, 't, Feat> {
    /// Raseterize the text and put it into the group context.
    fn rasterize_and_put_text(
        &mut self,
        _group_ctx: SvgTextBuilder,
        _abs_ref: &Fingerprint,
        _text: &TextItem,
    ) -> SvgTextBuilder {
        panic!("Rasterization is not enabled.")
    }
}

#[cfg(feature = "aggresive-browser-rasterization")]
impl<'m, 't, Feat: ExportFeature> RenderContext<'m, 't, Feat> {
    /// Raseterize the text and put it into the group context.
    fn rasterize_and_put_text(
        &mut self,
        mut group_ctx: SvgTextBuilder,
        abs_ref: &Fingerprint,
        text: &TextItem,
    ) -> SvgTextBuilder {
        use reflexo_vec2canvas::CanvasRenderSnippets;

        let font = self.get_font(&text.shape.font).unwrap();

        // upem is the unit per em defined in the font.
        let upem = font.units_per_em;

        group_ctx = text.shape.add_transform(self, group_ctx, upem);

        //

        let width = text.width();
        let mut _width = 0f32;
        let iter = text
            .render_glyphs(upem, &mut _width)
            .flat_map(|(pos, g)| font.get_glyph(g).map(|g| (pos, g.as_ref())));
        let scaled_width = width.0 * upem.0 / text.shape.size.0;
        let decender_adjust = (font.descender.0 * upem.0).abs();
        // .max(0.)
        let div_text = CanvasRenderSnippets::rasterize_text(
            abs_ref,
            iter,
            scaled_width,
            upem.0,
            decender_adjust,
            "#000",
        );

        group_ctx.content.push(SvgText::Plain(format!(
            r#"<foreignObject width="{:.1}" height="{:.1}" x="0" y="{:.1}">{div_text}</foreignObject>"#,
            scaled_width,
            upem.0 + decender_adjust,
            -decender_adjust,
        )));

        if self.should_render_text_element() {
            group_ctx.render_text_semantics_inner(
                &text.shape,
                &text.content.content,
                width,
                font.ascender,
                upem,
                self.should_aware_html_entity(),
            )
        }

        group_ctx
    }
}

impl<'m, 't, Feat: ExportFeature> RenderContext<'m, 't, Feat> {
    /// Render a text into the underlying context.
    fn render_text_inplace(
        &mut self,
        mut group_ctx: SvgTextBuilder,
        text: &TextItem,
    ) -> SvgTextBuilder {
        if text.shape.size.0 == 0. {
            return group_ctx;
        }

        let font = self.get_font(&text.shape.font).unwrap();

        // upem is the unit per em defined in the font.
        let upem = font.units_per_em;

        group_ctx = text.shape.add_transform(self, group_ctx, upem);

        let width = match (&group_ctx.text_fill, &group_ctx.text_stroke) {
            (fill, Some(stroke)) => {
                let mut width = 0f32;
                let fill = fill.clone();
                let stroke = stroke.clone();
                for (x, g) in text.render_glyphs(upem, &mut width) {
                    group_ctx.render_glyph_slow(x, font, g, fill.clone(), stroke.clone());
                }

                width
            }
            (None, None) => {
                let mut width = 0f32;
                for (x, g) in text.render_glyphs(upem, &mut width) {
                    group_ctx.render_glyph(self, x, font, g);
                }

                width
            }
            (Some(fill), None) => {
                // clip path rect
                let clip_id = fill.id.as_svg_id("pc");
                let fill_id = fill.id.as_svg_id("pf");

                // because the text is already scaled by the font size,
                // we need to scale it back to the original size.
                // todo: infinite multiplication
                let descender = font.descender.0 * upem.0;

                group_ctx.content.push(SvgText::Plain(format!(
                    r#"<clipPath id="{}" clipPathUnits="userSpaceOnUse">"#,
                    clip_id
                )));

                let mut width = 0f32;
                for (x, g) in text.render_glyphs(upem, &mut width) {
                    group_ctx.render_glyph(self, x, font, g);
                    group_ctx.content.push(SvgText::Plain("<path/>".into()));
                }

                group_ctx
                    .content
                    .push(SvgText::Plain(r#"</clipPath>"#.to_owned()));

                // clip path rect
                let scaled_width = width * upem.0 / text.shape.size.0;
                group_ctx.content.push(SvgText::Plain(format!(
                    r##"<rect fill="url(#{fill_id})" stroke="none" width="{:.1}" height="{:.1}" y="{:.1}" clip-path="url(#{})"/>"##,
                    scaled_width, upem.0, descender, clip_id
                )));

                // image glyphs
                let mut _width = 0f32;
                for (x, g) in text.render_glyphs(upem, &mut _width) {
                    let built = font.get_glyph(g);
                    if matches!(
                        built.map(Deref::deref),
                        Some(FlatGlyphItem::Outline(..)) | None
                    ) {
                        continue;
                    }
                    group_ctx.render_glyph(self, x, font, g);
                }

                width
            }
        };

        if self.should_render_text_element() {
            group_ctx.render_text_semantics_inner(
                &text.shape,
                &text.content.content,
                Scalar(width),
                font.ascender,
                upem,
            )
        }

        group_ctx
    }
}
