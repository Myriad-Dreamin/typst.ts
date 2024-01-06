// #[cfg(feature = "vector-bbox")]
// pub mod bbox;
pub mod incr;
pub mod ir;
pub(crate) mod lowering;
mod path2d;
#[cfg(feature = "rkyv")]
pub mod stream;
mod utils;
pub mod vm;

pub use ir::geom;
pub use lowering::{span_id_from_u64, span_id_to_u64, GlyphLowerBuilder};

#[cfg(feature = "rkyv")]
#[allow(dead_code)]
fn rkyv_assertions() {
    use ir::*;

    const _: () = assert!(core::mem::size_of::<()>() == 0);
    const _: () = assert!(core::mem::align_of::<()>() == 1);
    const _: () = assert!(core::mem::size_of::<bool>() == 1);
    const _: () = assert!(core::mem::align_of::<bool>() == 1);
    const _: () = assert!(core::mem::size_of::<u8>() == 1);
    const _: () = assert!(core::mem::align_of::<u8>() == 1);
    const _: () = assert!(core::mem::size_of::<u16>() == 2);
    const _: () = assert!(core::mem::align_of::<u16>() == 2);
    const _: () = assert!(core::mem::size_of::<u32>() == 4);
    const _: () = assert!(core::mem::align_of::<u32>() == 4);
    const _: () = assert!(core::mem::size_of::<u64>() == 8);
    const _: () = assert!(core::mem::align_of::<u64>() == 8);
    const _: () = assert!(core::mem::size_of::<i8>() == 1);
    const _: () = assert!(core::mem::align_of::<i8>() == 1);
    const _: () = assert!(core::mem::size_of::<i16>() == 2);
    const _: () = assert!(core::mem::align_of::<i16>() == 2);
    const _: () = assert!(core::mem::size_of::<i32>() == 4);
    const _: () = assert!(core::mem::align_of::<i32>() == 4);
    const _: () = assert!(core::mem::size_of::<i64>() == 8);
    const _: () = assert!(core::mem::align_of::<i64>() == 8);
    const _: () = assert!(core::mem::size_of::<f32>() == 4);
    const _: () = assert!(core::mem::align_of::<f32>() == 4);
    const _: () = assert!(core::mem::size_of::<f64>() == 8);
    const _: () = assert!(core::mem::align_of::<f64>() == 8);
    const _: () = assert!(core::mem::size_of::<char>() == 4);
    const _: () = assert!(core::mem::align_of::<char>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedSourceMappingNode>() == 16);
    const _: () = assert!(core::mem::align_of::<ArchivedSourceMappingNode>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedVecItem>() == 32);
    const _: () = assert!(core::mem::align_of::<ArchivedVecItem>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedFlatGlyphItem>() == 8);
    const _: () = assert!(core::mem::align_of::<ArchivedFlatGlyphItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedPatternItem>() == 40);
    const _: () = assert!(core::mem::align_of::<ArchivedPatternItem>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedModuleMetadata>() == 12);
    const _: () = assert!(core::mem::align_of::<ArchivedModuleMetadata>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedTextItem>() == 16);
    const _: () = assert!(core::mem::align_of::<ArchivedTextItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedTextItemContent>() == 16);
    const _: () = assert!(core::mem::align_of::<ArchivedTextItemContent>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedTransformedRef>() == 24);
    const _: () = assert!(core::mem::align_of::<ArchivedTransformedRef>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedGroupRef>() == 8);
    const _: () = assert!(core::mem::align_of::<ArchivedGroupRef>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedItemPack>() == 8);
    const _: () = assert!(core::mem::align_of::<ArchivedItemPack>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedFlatModule>() == 16);
    const _: () = assert!(core::mem::align_of::<ArchivedFlatModule>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedDefId>() == 8);
    const _: () = assert!(core::mem::align_of::<ArchivedDefId>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedAbsoluteRef>() == 24);
    const _: () = assert!(core::mem::align_of::<ArchivedAbsoluteRef>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedImage>() == 56);
    const _: () = assert!(core::mem::align_of::<ArchivedImage>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedImageItem>() == 12);
    const _: () = assert!(core::mem::align_of::<ArchivedImageItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedLinkItem>() == 16);
    const _: () = assert!(core::mem::align_of::<ArchivedLinkItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedPathItem>() == 28);
    const _: () = assert!(core::mem::align_of::<ArchivedPathItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedImageGlyphItem>() == 40);
    const _: () = assert!(core::mem::align_of::<ArchivedImageGlyphItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedOutlineGlyphItem>() == 20);
    const _: () = assert!(core::mem::align_of::<ArchivedOutlineGlyphItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedTextShape>() == 20);
    const _: () = assert!(core::mem::align_of::<ArchivedTextShape>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedPathStyle>() == 12);
    const _: () = assert!(core::mem::align_of::<ArchivedPathStyle>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedTransformItem>() == 8);
    const _: () = assert!(core::mem::align_of::<ArchivedTransformItem>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedIncrGlyphPack>() == 12);
    const _: () = assert!(core::mem::align_of::<ArchivedIncrGlyphPack>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedPage>() == 24);
    const _: () = assert!(core::mem::align_of::<ArchivedPage>() == 8);
    const _: () = assert!(core::mem::size_of::<ArchivedLayoutRegion>() == 20);
    const _: () = assert!(core::mem::align_of::<ArchivedLayoutRegion>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedBuildInfo>() == 16);
    const _: () = assert!(core::mem::align_of::<ArchivedBuildInfo>() == 4);
    const _: () = assert!(core::mem::size_of::<ArchivedColorItem>() == 4);
    const _: () = assert!(core::mem::align_of::<ArchivedColorItem>() == 1);
    const _: () = assert!(core::mem::size_of::<ArchivedGradientItem>() == 28);
    const _: () = assert!(core::mem::align_of::<ArchivedGradientItem>() == 4);
}
