use typst_ts_core::vector::ir::*;

macro_rules! layout {
    ($($prim:ty),* $(,)?) => {
        $(layout!(@one $prim);)*
    };
    (@one $prim:ty) => {{
        println!("const _: () = assert!(core::mem::size_of::<{}>() == {});", stringify!($prim), core::mem::size_of::<$prim>());
        println!("const _: () = assert!(core::mem::align_of::<{}>() == {});", stringify!($prim), core::mem::align_of::<$prim>());
    }};
}

fn main() {
    layout!(
        (),
        bool,
        u8,
        u16,
        u32,
        u64,
        // forbidden
        // u128,
        i8,
        i16,
        i32,
        i64,
        // forbidden
        // i128,
        f32,
        f64,
        char,
        ArchivedSourceMappingNode,
        ArchivedVecItem,
        ArchivedFlatGlyphItem,
        ArchivedPatternItem,
        ArchivedModuleMetadata,
        ArchivedTextItem,
        ArchivedTextItemContent,
        ArchivedTransformedRef,
        ArchivedGroupRef,
        ArchivedItemPack,
        ArchivedFlatModule,
        ArchivedDefId,
        ArchivedAbsoluteRef,
        ArchivedImage,
        ArchivedImageItem,
        ArchivedLinkItem,
        ArchivedPathItem,
        ArchivedImageGlyphItem,
        ArchivedOutlineGlyphItem,
        ArchivedTextShape,
        ArchivedPathStyle,
        ArchivedTransformItem,
        ArchivedIncrGlyphPack,
        ArchivedPage,
        ArchivedBuildInfo,
        // color
        ArchivedRgba8Item,
        ArchivedColor32Item,
        ArchivedColorSpace,
        ArchivedGradientItem,
        ArchivedGradientKind,
        ArchivedGradientStyle,
        // layout
        ArchivedLayoutRegionNode,
        ArchivedLayoutRegion,
    );
}
