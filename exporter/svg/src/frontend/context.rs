use siphasher::sip128::Hasher128;
use std::{collections::HashMap, hash::Hash, sync::Arc};

use typst_ts_core::{
    hash::{item_hash128, Fingerprint, FingerprintBuilder, FingerprintSipHasherBase},
    vector::{
        flat_ir::{FlatSvgItem, FlatTextItem, GroupRef, Module},
        flat_vm::{FlatIncrRenderVm, FlatRenderVm},
        ir::{
            self, BuildGlyph, FontIndice, FontRef, GlyphHashStablizer, GlyphPackBuilder, GlyphRef,
            ImmutStr, PathItem, StyleNs,
        },
        vm::GroupContext,
        vm::{RenderState, RenderVm},
    },
};

use crate::{
    backend::{
        BuildClipPath, BuildFillStyleClass, DynExportFeature, NotifyPaint, SvgTextBuilder,
        SvgTextNode,
    },
    ExportFeature, GlyphProvider,
};

/// Maps the style name to the style definition.
/// See [`StyleNs`].
pub(crate) type StyleDefMap = HashMap<(StyleNs, ImmutStr), String>;
/// Maps the clip path's data to the clip path id.
pub(crate) type ClipPathMap = HashMap<ImmutStr, Fingerprint>;
/// Maps the clip path's data to the clip path id.
pub(crate) type GradientMap = HashMap<ImmutStr, (u8, Fingerprint)>;

/// The task context for rendering svg items
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of SVG task.
pub struct RenderContext<'m, 't, Feat: ExportFeature> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    pub glyph_provider: GlyphProvider,

    #[cfg(feature = "flat-vector")]
    pub module: &'m Module,

    /// A fingerprint builder for generating unique id.
    pub(crate) fingerprint_builder: &'t mut FingerprintBuilder,

    /// Stores the glyphs used in the document.
    // todo: used in SvgItem rendering, but
    // unused in FlatSvgItem rendering, which is confusing.
    pub(crate) glyph_defs: &'t mut GlyphPackBuilder,
    /// Stores the style definitions used in the document.
    pub(crate) style_defs: &'t mut StyleDefMap,
    /// Stores the clip paths used in the document.
    pub(crate) clip_paths: &'t mut ClipPathMap,
    /// Stores the clip paths used in the document.
    pub(crate) gradients: &'t mut GradientMap,

    /// See [`ExportFeature`].
    pub should_render_text_element: bool,
    /// See [`ExportFeature`].
    pub should_attach_debug_info: bool,
    /// See [`ExportFeature`].
    pub use_stable_glyph_id: bool,

    pub _feat_phantom: std::marker::PhantomData<Feat>,
    #[cfg(not(feature = "flat-vector"))]
    pub _m_phantom: std::marker::PhantomData<&'m ()>,
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

impl<'m, 't, Feat: ExportFeature> BuildGlyph for RenderContext<'m, 't, Feat> {
    fn build_font(&mut self, font: &typst::font::Font) -> FontRef {
        self.glyph_defs.build_font(font)
    }

    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> GlyphRef {
        self.glyph_defs.build_glyph(glyph)
    }
}

impl<'m, 't, Feat: ExportFeature> GlyphHashStablizer for RenderContext<'m, 't, Feat> {
    fn stablize_hash(&mut self, glyph: &GlyphRef) -> Fingerprint {
        Fingerprint::from_u128(item_hash128(
            &self.module.glyphs[glyph.glyph_idx as usize].1,
        ))
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
        if let Some(id) = self.clip_paths.get(&path.d) {
            return *id;
        }

        let fingerprint = self.fingerprint_builder.resolve(path);
        self.clip_paths.insert(path.d.clone(), fingerprint);
        fingerprint
    }
}

impl<'m, 't, Feat: ExportFeature> NotifyPaint for RenderContext<'m, 't, Feat> {
    fn notify_paint(&mut self, url_ref: ImmutStr) -> (u8, Fingerprint) {
        if let Some(f) = self.gradients.get(&url_ref) {
            return *f;
        }

        // url(#ghash)
        if !url_ref.starts_with("url(#g") || !url_ref.ends_with(')') {
            panic!("Invalid url reference: {}", url_ref);
        }

        let id = url_ref.trim_start_matches("url(#g").trim_end_matches(')');
        let id = Fingerprint::try_from_str(id).unwrap();

        let kind = match self.get_item(&id) {
            Some(FlatSvgItem::Gradient(g)) => &g.kind,
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

        self.gradients.insert(url_ref, (kind, id));
        (kind, id)
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

        g.with_text_shape(self, &text.shape, &Fingerprint::from_u128(k), state);
        g.attach_debug_info(self, text.content.span_id);

        g
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
        g.with_text_shape(self, &text.shape, value, state);
        g
    }
}

impl<'m, 't, Feat: ExportFeature> FlatIncrRenderVm<'m> for RenderContext<'m, 't, Feat> {}
