use siphasher::sip128::Hasher128;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use typst_ts_core::{
    hash::{Fingerprint, FingerprintBuilder, FingerprintSipHasherBase},
    vector::{
        flat_ir::{FlatSvgItem, FlatTextItem, GroupRef, Module},
        flat_vm::{FlatGroupContext, FlatIncrRenderVm, FlatRenderVm},
        ir::{
            self, BuildGlyph, FontIndice, FontRef, GlyphHashStablizer, GlyphIndice, GlyphItem,
            GlyphPackBuilder, GlyphRef, ImmutStr, PathItem, PathStyle, Scalar, StyleNs,
        },
        vm::GroupContext,
        vm::{RenderState, RenderVm},
    },
};

use crate::{
    backend::{
        BuildClipPath, BuildFillStyleClass, DynExportFeature, NotifyPaint, SvgText, SvgTextBuilder,
        SvgTextNode,
    },
    utils::MemorizeFree,
    ExportFeature, GlyphProvider,
};

use super::HasStatefulFill;

/// Maps the style name to the style definition.
/// See [`StyleNs`].
pub(crate) type StyleDefMap = HashMap<(StyleNs, ImmutStr), String>;
/// Maps paint fill id to the paint fill's data.
pub(crate) type PaintFillMap = HashMap<ImmutStr, (u8, Fingerprint, Option<bool>)>;

/// The task context for rendering svg items
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of SVG task.
pub struct RenderContext<'m, 't, Feat: ExportFeature> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    pub glyph_provider: GlyphProvider,

    pub module: &'m Module,

    /// A fingerprint builder for generating unique id.
    pub(crate) fingerprint_builder: &'t mut FingerprintBuilder,

    /// Stores the glyphs used in the document.
    // todo: used in SvgItem rendering, but
    // unused in FlatSvgItem rendering, which is confusing.
    pub(crate) glyph_defs: &'t mut GlyphPackBuilder,
    /// Stores the style definitions used in the document.
    pub(crate) style_defs: &'t mut StyleDefMap,
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
    fn enable_inlined_svg(&self) -> bool {
        Feat::ENABLE_INLINED_SVG
    }

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
        self.module.fonts.get(value.idx as usize)
    }
}

impl<'m, 't, Feat: ExportFeature> GlyphIndice<'m> for RenderContext<'m, 't, Feat> {
    fn get_glyph(&self, g: &GlyphRef) -> Option<&'m ir::GlyphItem> {
        self.module.glyphs.get(g.glyph_idx as usize).map(|v| &v.1)
    }
}

impl<'m, 't, Feat: ExportFeature> BuildGlyph for RenderContext<'m, 't, Feat> {
    fn build_font(&mut self, font: &typst::text::Font) -> FontRef {
        self.glyph_defs.build_font(font)
    }

    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> GlyphRef {
        self.glyph_defs.build_glyph(glyph)
    }
}

impl<'m, 't, Feat: ExportFeature> GlyphHashStablizer for RenderContext<'m, 't, Feat> {
    fn stablize_hash(&mut self, glyph: &GlyphRef) -> Fingerprint {
        let glyph = &self.module.glyphs[glyph.glyph_idx as usize].1;
        glyph.get_fingerprint()
    }
}

impl<'m, 't, Feat: ExportFeature> BuildFillStyleClass for RenderContext<'m, 't, Feat> {
    fn build_fill_style_class(&mut self, fill: ImmutStr) -> String {
        // insert fill definition
        let fill_id = format!(r#"f{}"#, fill.trim_start_matches('#'));
        let fill_key = (StyleNs::Fill, fill.clone());
        self.style_defs
            .entry(fill_key)
            .or_insert_with(|| format!(r#"g.{} {{ --glyph_fill: {}; }} "#, fill_id, fill));

        fill_id
    }
}

impl<'m, 't, Feat: ExportFeature> BuildClipPath for RenderContext<'m, 't, Feat> {
    fn build_clip_path(&mut self, path: &PathItem) -> Fingerprint {
        self.fingerprint_builder.resolve(path)
    }
}

#[comemo::memoize]
fn has_stateful_fill<'m, 't, Feat: ExportFeature>(
    ctx: &MemorizeFree<RenderContext<'m, 't, Feat>>,
    x: &Fingerprint,
) -> bool {
    let Some(item) = ctx.0.get_item(x) else {
        // overestimated
        return true;
    };

    use FlatSvgItem::*;
    match item {
        Gradient(..) | Pattern(..) => true,
        Image(..) | Link(..) | ContentHint(..) | None => false,
        Item(t) => has_stateful_fill(ctx, &t.1),
        Group(g, ..) => g.0.iter().any(|(_, x)| has_stateful_fill(ctx, x)),
        Path(p) => p.styles.iter().any(|s| match s {
            PathStyle::Fill(color) | PathStyle::Stroke(color) => color.starts_with('@'),
            _ => false,
        }),
        Text(p) => p.shape.fill.starts_with('@'),
    }
}

impl<'m, 't, Feat: ExportFeature> HasStatefulFill for RenderContext<'m, 't, Feat> {
    fn has_stateful_fill(&self, a: &Fingerprint) -> bool {
        has_stateful_fill(&MemorizeFree(self), a)
    }
}

impl<'m, 't, Feat: ExportFeature> NotifyPaint for RenderContext<'m, 't, Feat> {
    fn notify_paint(&mut self, url_ref: ImmutStr) -> (u8, Fingerprint, Option<bool>) {
        let mp = if url_ref.starts_with("@g") {
            &mut self.gradients
        } else if url_ref.starts_with("@p") {
            &mut self.patterns
        } else {
            panic!("Invalid url reference: {}", url_ref);
        };

        if let Some(f) = mp.get(&url_ref) {
            return *f;
        }

        // url(#ghash)
        if url_ref.starts_with("@g") {
            let id = url_ref.trim_start_matches("@g");
            let id = Fingerprint::try_from_str(id).unwrap();

            let (kind, relative_to_self) = match self.get_item(&id) {
                Some(FlatSvgItem::Gradient(g)) => (&g.kind, g.relative_to_self),
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

            self.gradients.insert(url_ref, (kind, id, relative_to_self));
            (kind, id, relative_to_self)
        } else if url_ref.starts_with("@p") {
            let id = url_ref.trim_start_matches("@p");
            let id = Fingerprint::try_from_str(id).unwrap();

            let relative_to_self = match self.get_item(&id) {
                Some(FlatSvgItem::Pattern(g)) => g.relative_to_self,
                _ => {
                    // #[cfg(debug_assertions)]
                    panic!("Invalid pattern reference: {}", id.as_svg_id("p"));
                }
            };

            let kind = b'p';

            self.patterns.insert(url_ref, (kind, id, relative_to_self));
            (kind, id, relative_to_self)
        } else {
            panic!("Invalid url reference: {}", url_ref);
        }
    }
}

/// Example of how to implement a RenderVm.
impl<'m, 't, Feat: ExportFeature> RenderVm for RenderContext<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = Arc<SvgTextNode>;
    type Group = SvgTextBuilder;

    fn start_group(&mut self) -> Self::Group {
        Self::Group {
            attributes: vec![],
            text_fill: None,
            content: Vec::with_capacity(1),
        }
    }

    fn start_frame(&mut self, _group: &ir::GroupItem) -> Self::Group {
        let mut g = self.start_group();
        g.attributes.push(("class", "typst-group".to_owned()));
        g
    }

    fn start_text(&mut self, state: RenderState, text: &ir::TextItem) -> Self::Group {
        let mut g = self.start_group();

        let mut k = FingerprintSipHasherBase::new();
        text.font.hash(&mut k);
        text.content.glyphs.hash(&mut k);
        text.shape.hash(&mut k);
        let k = k.finish128().as_u128();

        let upem = Scalar(text.font.units_per_em() as f32);

        g.with_text_shape(
            self,
            upem,
            &text.shape,
            &state.at(&Fingerprint::from_u128(k)),
            state,
        );
        g.attach_debug_info(self, text.content.span_id);

        g
    }

    /// Render a text into the underlying context.
    fn render_text(&mut self, _state: RenderState, _text: &ir::TextItem) -> Self::Resultant {
        unreachable!();
    }
}

impl<'m, 't, Feat: ExportFeature> FlatRenderVm<'m> for RenderContext<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = Arc<SvgTextNode>;
    type Group = SvgTextBuilder;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&mut self, v: &Fingerprint) -> Self::Group {
        Self::Group {
            attributes: vec![("data-tid", v.as_svg_id("g"))],
            text_fill: None,
            content: Vec::with_capacity(1),
        }
    }

    fn start_flat_frame(&mut self, value: &Fingerprint, _group: &GroupRef) -> Self::Group {
        let mut g = self.start_flat_group(value);
        g.attributes.push(("class", "typst-group".to_owned()));
        g
    }

    fn start_flat_text(
        &mut self,
        state: RenderState,
        value: &Fingerprint,
        text: &FlatTextItem,
    ) -> Self::Group {
        let mut g = self.start_flat_group(value);

        let font = self.get_font(&text.font).unwrap();
        let upem = font.unit_per_em;

        g.with_text_shape(self, upem, &text.shape, &state.at(value), state);
        g
    }

    /// Render a text into the underlying context.
    fn render_flat_text(
        &mut self,
        _state: RenderState,
        group_ctx: Self::Group,
        abs_ref: &Fingerprint,
        text: &FlatTextItem,
    ) -> Self::Group {
        if self.should_rasterize_text() {
            self.rasterize_and_put_text(group_ctx, abs_ref, text)
        } else {
            self.render_flat_text_inplace(group_ctx, text)
        }
    }
}

impl<'m, 't, Feat: ExportFeature> FlatIncrRenderVm<'m> for RenderContext<'m, 't, Feat> {}

#[cfg(not(feature = "aggresive-browser-rasterization"))]
impl<'m, 't, Feat: ExportFeature> RenderContext<'m, 't, Feat> {
    /// Raseterize the text and put it into the group context.
    fn rasterize_and_put_text(
        &mut self,
        _group_ctx: SvgTextBuilder,
        _abs_ref: &Fingerprint,
        _text: &FlatTextItem,
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
        text: &FlatTextItem,
    ) -> SvgTextBuilder {
        use typst_ts_canvas_exporter::CanvasRenderSnippets;

        let font = self.get_font(&text.font).unwrap();

        // upem is the unit per em defined in the font.
        let upem = font.unit_per_em;

        group_ctx = text.shape.add_transform(self, group_ctx, upem);

        //

        let width = text.width();
        let mut _width = 0f32;
        let iter = text
            .render_glyphs(upem, &mut _width)
            .flat_map(|(pos, g)| self.get_glyph(g).map(|g| (pos, g)));
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
    fn render_flat_text_inplace(
        &mut self,
        mut group_ctx: SvgTextBuilder,
        text: &FlatTextItem,
    ) -> SvgTextBuilder {
        let font = self.get_font(&text.font).unwrap();

        // upem is the unit per em defined in the font.
        let upem = font.unit_per_em;

        group_ctx = text.shape.add_transform(self, group_ctx, upem);

        let width = if let Some(fill) = &group_ctx.text_fill {
            // clip path rect
            let clip_id = fill.as_svg_id("tc");
            let fill_id = fill.as_svg_id("tf");

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
                group_ctx.render_glyph_ref(self, x, g);
                group_ctx.content.push(SvgText::Plain("<path/>".into()));
            }

            group_ctx
                .content
                .push(SvgText::Plain(r#"</clipPath>"#.to_owned()));

            // clip path rect
            let scaled_width = width * upem.0 / text.shape.size.0;
            group_ctx.content.push(SvgText::Plain(format!(
                r##"<rect fill="url(#{})" width="{:.1}" height="{:.1}" y="{:.1}" clip-path="url(#{})"/>"##,
                fill_id, scaled_width, upem.0, descender, clip_id
            )));

            // image glyphs
            let mut _width = 0f32;
            for (x, g) in text.render_glyphs(upem, &mut _width) {
                let built = self.get_glyph(g);
                if matches!(
                    built,
                    Some(GlyphItem::Outline(..) | GlyphItem::Raw(..)) | None
                ) {
                    continue;
                }
                group_ctx.render_glyph_ref(self, x, g);
            }

            width
        } else {
            let mut width = 0f32;
            for (x, g) in text.render_glyphs(upem, &mut width) {
                group_ctx.render_glyph_ref(self, x, g);
            }

            width
        };

        if self.should_render_text_element() {
            group_ctx.render_text_semantics_inner(
                &text.shape,
                &text.content.content,
                Scalar(width),
                font.ascender,
                upem,
                self.should_aware_html_entity(),
            )
        }

        group_ctx
    }
}
