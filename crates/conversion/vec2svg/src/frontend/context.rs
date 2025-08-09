use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use reflexo::{
    hash::{Fingerprint, FingerprintBuilder},
    vector::{
        ir::{
            self, FlatGlyphItem, FontIndice, FontRef, GroupRef, ImmutStr, Module, PathItem, Scalar,
            TextItem, Transform, VecItem,
        },
        vm::{GroupContext, IncrRenderVm, RenderVm},
    },
};
use reflexo_typst2vec::ir::Axes;

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
    /// Stores the gradients used in the document.
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

impl<Feat: ExportFeature> DynExportFeature for RenderContext<'_, '_, Feat> {
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

impl<'m, Feat: ExportFeature> FontIndice<'m> for RenderContext<'m, '_, Feat> {
    fn get_font(&self, value: &FontRef) -> Option<&'m ir::FontItem> {
        self.module.fonts.get(value.idx as usize).inspect(|e| {
            // canary check
            if e.hash != value.hash {
                panic!("Invalid font reference: {value:?}");
            }
        })
    }
}

impl<Feat: ExportFeature> BuildClipPath for RenderContext<'_, '_, Feat> {
    fn build_clip_path(&mut self, path: &PathItem) -> Fingerprint {
        self.fingerprint_builder.resolve(path)
    }
}

impl<Feat: ExportFeature> NotifyPaint for RenderContext<'_, '_, Feat> {
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
            panic!("Invalid url reference: {url_ref}");
        }
    }
}

/// Example of how to implement a FlatRenderVm.
impl<'m, Feat: ExportFeature> RenderVm<'m> for RenderContext<'m, '_, Feat> {
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
        _abs_ref: &Fingerprint,
        text: &TextItem,
    ) -> Self::Group {
        self.render_text_inplace(group_ctx, text)
    }
}

impl<'m, Feat: ExportFeature> IncrRenderVm<'m> for RenderContext<'m, '_, Feat> {}

impl<Feat: ExportFeature> RenderContext<'_, '_, Feat> {
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

        let size = match (&group_ctx.text_fill, &group_ctx.text_stroke) {
            (fill, Some(stroke)) => {
                let mut size = Axes { x: 0f32, y: 0f32 };
                let fill = fill.clone();
                let stroke = stroke.clone();
                for (s, g) in text.render_glyphs(upem, &mut size) {
                    group_ctx.render_glyph_slow(s, font, g, fill.clone(), stroke.clone());
                }

                size
            }
            (None, None) => {
                let mut size = Axes { x: 0f32, y: 0f32 };
                for (s, g) in text.render_glyphs(upem, &mut size) {
                    group_ctx.render_glyph(self, s, font, g);
                }

                size
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
                    r#"<clipPath id="{clip_id}" clipPathUnits="userSpaceOnUse">"#
                )));

                let mut size = Axes { x: 0f32, y: 0f32 };
                for (s, g) in text.render_glyphs(upem, &mut size) {
                    group_ctx.render_glyph(self, s, font, g);
                    group_ctx.content.push(SvgText::Plain("<path/>".into()));
                }

                group_ctx
                    .content
                    .push(SvgText::Plain(r#"</clipPath>"#.to_owned()));

                // clip path rect
                let scaled_width = size.x * upem.0 / text.shape.size.0;
                group_ctx.content.push(SvgText::Plain(format!(
                    r##"<rect fill="url(#{fill_id})" stroke="none" width="{:.1}" height="{:.1}" y="{:.1}" clip-path="url(#{})"/>"##,
                    scaled_width, upem.0, descender, clip_id
                )));

                // image glyphs
                let mut _size = Axes { x: 0f32, y: 0f32 };

                for (s, g) in text.render_glyphs(upem, &mut _size) {
                    let built = font.get_glyph(g);
                    if matches!(
                        built.map(Deref::deref),
                        Some(FlatGlyphItem::Outline(..)) | None
                    ) {
                        continue;
                    }
                    group_ctx.render_glyph(self, s, font, g);
                }

                size
            }
        };

        if self.should_render_text_element() {
            group_ctx.render_text_semantics_inner(
                &text.shape,
                &text.content.content,
                Scalar(size.x),
                font.ascender,
                upem,
            )
        }

        group_ctx
    }
}
