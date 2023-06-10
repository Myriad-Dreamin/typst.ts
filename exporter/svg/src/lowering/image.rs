use std::sync::Arc;

use crate::ir::{ImageItem, SvgItem};
use crate::{RenderFeature, SvgRenderTask};
use typst::geom::Size;
use typst::image::Image;

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Lower a raster or SVG image into item.
    // todo: error handling
    pub(super) fn lower_image(&mut self, image: &Image, size: Size) -> SvgItem {
        SvgItem::Image(Arc::new(ImageItem {
            image: image.clone(),
            size,
        }))
    }
}
