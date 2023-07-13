use std::sync::Arc;

use comemo::Prehashed;

use super::flat_ir::Module;
use super::flat_vm::FlatGroupContext;
use super::flat_vm::FlatRenderVm;
use super::ir::BuildGlyph;
use super::ir::DefId;
use super::ir::FingerprintBuilder;
use super::ir::GlyphMapping;
use super::ir::Transform;
use super::vm::RenderVm;
use crate::font::GlyphProvider;

use super::flat_ir;
use super::ir::{self, Abs, AbsoluteRef, Axes, Ratio, Scalar};
use super::sk;
use super::vm::{GroupContext, TransformContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rect {
    pub w: Abs,
    pub h: Abs,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineBBoxRef {
    pub bbox: BBox,
    pub abs_ref: AbsoluteRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BBoxRepr {
    Group(Transform, Arc<[(ir::Point, InlineBBoxRef)]>),
    Clip(InlineBBoxRef),
    Indirect(InlineBBoxRef),
    Node(AbsoluteRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BBox(Arc<Prehashed<BBoxRepr>>);

impl BBox {
    #[comemo::memoize]
    pub fn realize(&self, _ts: Transform) -> Rect {
        todo!()
    }
}

/// Task to create bbox with vector IR
/// The 'm lifetime is the lifetime of the module which stores the frame data.
/// The 't lifetime is the lifetime of task.
pub struct BBoxTask<'m, 't> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    pub glyph_provider: GlyphProvider,

    #[cfg(feature = "flat-vector")]
    pub module: &'m Module,

    /// A fingerprint builder for generating unique id.
    pub(crate) fingerprint_builder: &'t mut FingerprintBuilder,

    /// Stores the glyphs used in the document.
    pub(crate) glyph_defs: &'t mut GlyphMapping,

    #[cfg(not(feature = "flat-vector"))]
    pub _m_phantom: std::marker::PhantomData<&'m ()>,
}

/// A builder for [`BBox`].
pub struct BBoxBuilder {
    pub ts: sk::Transform,
    pub clipper: Option<ir::PathItem>,
    pub inner: Vec<(ir::Point, InlineBBoxRef)>,
}

impl From<BBoxBuilder> for Arc<BBox> {
    fn from(s: BBoxBuilder) -> Self {
        Arc::new(BBox(Arc::new(Prehashed::new(BBoxRepr::Group(
            s.ts.into(),
            s.inner.into(),
        )))))
    }
}

/// Internal methods for [`BBoxBuilder`].
impl BBoxBuilder {
    pub fn render_glyph_ref_inner(&mut self, _pos: Scalar, _glyph: &AbsoluteRef) {
        // let glyph_data = self.t.module.glyphs[glyph.id.0 as usize].1.clone();
        // self.inner.push((
        //     ir::Point::new(pos, Scalar(0.)),
        //     Arc::new(Box::new(Rect {
        //         fill: self.fill.clone().unwrap(),
        //         glyph_data,
        //     })),
        // ))
        todo!()
    }
}

/// See [`TransformContext`].
impl<C> TransformContext<C> for BBoxBuilder {
    fn transform_matrix(mut self, _ctx: &mut C, m: &ir::Transform) -> Self {
        let sub_ts: sk::Transform = (*m).into();
        self.ts = self.ts.post_concat(sub_ts);
        self
    }

    fn transform_translate(mut self, _ctx: &mut C, matrix: Axes<Abs>) -> Self {
        self.ts = self.ts.post_translate(matrix.x.0, matrix.y.0);
        self
    }

    fn transform_scale(mut self, _ctx: &mut C, x: Ratio, y: Ratio) -> Self {
        self.ts = self.ts.post_scale(x.0, y.0);
        self
    }

    fn transform_rotate(self, _ctx: &mut C, _matrix: Scalar) -> Self {
        todo!()
    }

    fn transform_skew(mut self, _ctx: &mut C, matrix: (Ratio, Ratio)) -> Self {
        self.ts = self.ts.post_concat(sk::Transform {
            sx: 1.,
            sy: 1.,
            kx: matrix.0 .0,
            ky: matrix.1 .0,
            tx: 0.,
            ty: 0.,
        });
        self
    }

    fn transform_clip(mut self, _ctx: &mut C, matrix: &ir::PathItem) -> Self {
        self.clipper = Some(matrix.clone());
        self
    }
}

/// See [`GroupContext`].
impl<C: BuildGlyph + RenderVm<Resultant = Arc<BBox>>> GroupContext<C> for BBoxBuilder {
    fn render_item_at(&mut self, ctx: &mut C, pos: ir::Point, item: &ir::SvgItem) {
        let ts = self.ts;
        self.ts = ts.post_translate(pos.x.0, pos.y.0);
        let _bbox = ctx.render_item(item);
        self.ts = ts;
        todo!()
    }

    fn render_glyph(&mut self, ctx: &mut C, pos: Scalar, glyph: &ir::GlyphItem) {
        let glyph_ref = ctx.build_glyph(glyph);
        self.render_glyph_ref_inner(pos, &glyph_ref)
    }

    fn render_path(&mut self, _ctx: &mut C, _path: &ir::PathItem) {
        // self.inner.push((
        //     ir::Point::default(),
        //     Arc::new(Box::new(PathElem {
        //         path_data: path.clone(),
        //     })),
        // ))
    }

    fn render_image(&mut self, _ctx: &mut C, _image_item: &ir::ImageItem) {
        // self.inner.push((
        //     ir::Point::default(),
        //     Arc::new(Box::new(Rect {
        //         image_data: image_item.clone(),
        //     })),
        // ))
    }
}

/// See [`FlatGroupContext`].
impl<'m, C: FlatRenderVm<'m, Resultant = Arc<BBox>>> FlatGroupContext<C> for BBoxBuilder {
    fn render_item_ref_at(&mut self, ctx: &mut C, pos: ir::Point, item: &AbsoluteRef) {
        let ts = self.ts;
        self.ts = ts.post_translate(pos.x.0, pos.y.0);
        let _t = ctx.render_flat_item(item);
        self.ts = ts;
    }

    fn render_glyph_ref(&mut self, _ctx: &mut C, pos: Scalar, glyph: &AbsoluteRef) {
        self.render_glyph_ref_inner(pos, glyph)
    }
}

impl<'m, 't> RenderVm for BBoxTask<'m, 't> {
    type Resultant = Arc<BBox>;
    type Group = BBoxBuilder;

    fn start_group(&mut self) -> Self::Group {
        Self::Group {
            ts: sk::Transform::identity(),
            clipper: None,
            inner: vec![],
        }
    }
}

impl<'m, 't> FlatRenderVm<'m> for BBoxTask<'m, 't> {
    type Resultant = Arc<BBox>;
    type Group = BBoxBuilder;

    fn get_item(&self, value: &AbsoluteRef) -> Option<&'m flat_ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&mut self, _v: &AbsoluteRef) -> Self::Group {
        self.start_group()
    }
}

impl BuildGlyph for BBoxTask<'_, '_> {
    fn build_glyph(&mut self, glyph: &ir::GlyphItem) -> AbsoluteRef {
        if let Some(id) = self.glyph_defs.get(glyph) {
            return id.clone();
        }

        let id = DefId(self.glyph_defs.len() as u64);

        let fingerprint = self.fingerprint_builder.resolve(glyph);
        let abs_ref = AbsoluteRef { fingerprint, id };
        self.glyph_defs.insert(glyph.clone(), abs_ref.clone());
        abs_ref
    }
}
