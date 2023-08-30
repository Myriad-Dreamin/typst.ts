use std::{collections::HashMap, sync::Arc};

use typst_ts_core::{
    hash::{item_hash128, Fingerprint, FingerprintBuilder},
    vector::{
        flat_ir::{FlatSvgItem, FlatTextItem, GroupRef, Module},
        flat_vm::{FlatIncrRenderVm, FlatRenderVm},
        ir::{
            self, BuildGlyph, GlyphHashStablizer, GlyphPackBuilder, GlyphRef, ImmutStr, PathItem,
            StyleNs,
        },
        vm::GroupContext,
        vm::RenderVm,
    },
};

use crate::{
    backend::{BuildClipPath, BuildFillStyleClass, DynExportFeature, SvgTextBuilder, SvgTextNode},
    ExportFeature, GlyphProvider,
};

/// Maps the style name to the style definition.
/// See [`StyleNs`].
pub(crate) type StyleDefMap = HashMap<(StyleNs, ImmutStr), String>;
/// Maps the clip path's data to the clip path id.
pub(crate) type ClipPathMap = HashMap<ImmutStr, Fingerprint>;

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

impl<'m, 't, Feat: ExportFeature> BuildGlyph for RenderContext<'m, 't, Feat> {
    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> GlyphRef {
        self.glyph_defs.build_glyph(glyph).0
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

/// Example of how to implement a RenderVm.
impl<'m, 't, Feat: ExportFeature> RenderVm for RenderContext<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = Arc<SvgTextNode>;
    type Group = SvgTextBuilder;

    fn start_group(&mut self) -> Self::Group {
        Self::Group {
            attributes: vec![],
            content: Vec::with_capacity(1),
        }
    }

    fn start_frame(&mut self, _group: &ir::GroupItem) -> Self::Group {
        let mut g = self.start_group();
        g.attributes.push(("class", "typst-group".to_owned()));
        g
    }

    fn start_text(&mut self, text: &ir::TextItem) -> Self::Group {
        let mut g = self.start_group();
        g.with_text_shape(self, &text.shape);
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
            content: Vec::with_capacity(1),
        }
    }

    fn start_flat_frame(&mut self, value: &Fingerprint, _group: &GroupRef) -> Self::Group {
        let mut g = self.start_flat_group(value);
        g.attributes.push(("class", "typst-group".to_owned()));
        g
    }

    fn start_flat_text(&mut self, value: &Fingerprint, text: &FlatTextItem) -> Self::Group {
        let mut g = self.start_flat_group(value);
        g.with_text_shape(self, &text.shape);
        g
    }
}

impl<'m, 't, Feat: ExportFeature> FlatIncrRenderVm<'m> for RenderContext<'m, 't, Feat> {}
