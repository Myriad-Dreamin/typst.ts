use std::sync::Arc;

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
pub use reflexo::vector::ir::*;

use reflexo::hash::{item_hash128, Fingerprint};
use reflexo::typst::Bytes;
use typst::foundations::Smart;
use typst::layout::{
    Abs as TypstAbs, Angle as TypstAngle, Axes as TypstAxes, Point as TypstPoint,
    Ratio as TypstRatio, Transform as TypstTransform,
};
use typst::text::Font;
use typst::utils::Scalar as TypstScalar;
use typst::visualize::{ExchangeFormat, ImageFormat, ImageKind, RasterFormat, VectorFormat};

use crate::hash::typst_affinite_hash;
use crate::{FromTypst, IntoTypst, TryFromTypst};

pub trait ImageExt {
    fn data(&self) -> &Bytes;
}

impl ImageExt for typst::visualize::Image {
    fn data(&self) -> &Bytes {
        match self.kind() {
            typst::visualize::ImageKind::Raster(raster_image) => raster_image.data(),
            typst::visualize::ImageKind::Svg(svg_image) => svg_image.data(),
        }
    }
}

impl FromTypst<Rgba8Item> for typst::visualize::Color {
    fn from_typst(v: Rgba8Item) -> Self {
        typst::visualize::Color::from_u8(v.r, v.g, v.b, v.a)
    }
}

impl FromTypst<typst::visualize::ColorSpace> for ColorSpace {
    fn from_typst(value: typst::visualize::ColorSpace) -> Self {
        use typst::visualize::ColorSpace::*;
        match value {
            Oklab => Self::Oklab,
            Oklch => Self::Oklch,
            Srgb => Self::Srgb,
            D65Gray => Self::D65Gray,
            LinearRgb => Self::LinearRgb,
            Hsl => Self::Hsl,
            Hsv => Self::Hsv,
            Cmyk => Self::Cmyk,
        }
    }
}

impl TryFromTypst<ColorSpace> for typst::visualize::ColorSpace {
    type Error = ();

    fn try_from_typst(value: ColorSpace) -> Result<Self, ()> {
        use typst::visualize::ColorSpace::*;
        Ok(match value {
            ColorSpace::Luma => return Err(()),
            ColorSpace::Oklab => Oklab,
            ColorSpace::Oklch => Oklch,
            ColorSpace::Srgb => Srgb,
            ColorSpace::D65Gray => D65Gray,
            ColorSpace::LinearRgb => LinearRgb,
            ColorSpace::Hsl => Hsl,
            ColorSpace::Hsv => Hsv,
            ColorSpace::Cmyk => Cmyk,
        })
    }
}

impl FromTypst<TypstScalar> for Scalar {
    fn from_typst(scalar: TypstScalar) -> Self {
        Self(scalar.get() as f32)
    }
}

impl FromTypst<Scalar> for TypstScalar {
    fn from_typst(scalar: Scalar) -> Self {
        <TypstScalar as std::convert::From<f64>>::from(scalar.0 as f64)
    }
}

impl FromTypst<TypstRatio> for Scalar {
    fn from_typst(ratio: TypstRatio) -> Self {
        Self(ratio.get() as f32)
    }
}

impl FromTypst<TypstAbs> for Scalar {
    fn from_typst(ratio: TypstAbs) -> Self {
        Self(ratio.to_pt() as f32)
    }
}

impl FromTypst<TypstAngle> for Scalar {
    fn from_typst(scalar: TypstAngle) -> Self {
        Self(scalar.to_rad() as f32)
    }
}

impl<U, T> FromTypst<TypstAxes<U>> for Axes<T>
where
    U: IntoTypst<T>,
{
    fn from_typst(typst_axes: TypstAxes<U>) -> Self {
        Self {
            x: typst_axes.x.into_typst(),
            y: typst_axes.y.into_typst(),
        }
    }
}

impl<T, U> FromTypst<Axes<T>> for TypstAxes<U>
where
    T: IntoTypst<U>,
{
    fn from_typst(axes: Axes<T>) -> Self {
        Self {
            x: axes.x.into_typst(),
            y: axes.y.into_typst(),
        }
    }
}

impl FromTypst<TypstPoint> for Point {
    fn from_typst(p: TypstPoint) -> Self {
        Self {
            x: p.x.into_typst(),
            y: p.y.into_typst(),
        }
    }
}

impl FromTypst<TypstTransform> for Transform {
    fn from_typst(typst_transform: TypstTransform) -> Self {
        Self {
            sx: typst_transform.sx.into_typst(),
            ky: typst_transform.ky.into_typst(),
            kx: typst_transform.kx.into_typst(),
            sy: typst_transform.sy.into_typst(),
            tx: typst_transform.tx.into_typst(),
            ty: typst_transform.ty.into_typst(),
        }
    }
}

impl FromTypst<Font> for FontItem {
    fn from_typst(font: Font) -> Self {
        let hash = reflexo::hash::hash32(&font);
        let fingerprint = Fingerprint::from_u128(item_hash128(&font));

        let metrics = font.metrics();
        Self {
            fingerprint,
            hash,
            family: font.info().family.clone().into(),
            cap_height: Scalar(metrics.cap_height.get() as f32),
            ascender: Scalar(metrics.ascender.get() as f32),
            descender: Scalar(metrics.descender.get() as f32),
            units_per_em: Scalar(font.units_per_em() as f32),
            vertical: false, // todo: check vertical
            glyphs: Vec::new(),
            glyph_cov: bitvec::vec::BitVec::new(),
        }
    }
}

/// Collect image data from [`typst::visualize::Image`].
impl FromTypst<typst::visualize::Image> for Image {
    fn from_typst(image: typst::visualize::Image) -> Self {
        let format = match image.format() {
            ImageFormat::Raster(RasterFormat::Exchange(ExchangeFormat::Jpg)) => "jpeg",
            ImageFormat::Raster(RasterFormat::Exchange(ExchangeFormat::Png)) => "png",
            ImageFormat::Raster(RasterFormat::Exchange(ExchangeFormat::Webp)) => "webp",
            ImageFormat::Raster(RasterFormat::Exchange(ExchangeFormat::Gif)) => "gif",
            ImageFormat::Raster(RasterFormat::Pixel(..)) => "png",
            ImageFormat::Vector(VectorFormat::Svg) => "svg+xml",
        };
        let attrs = {
            let mut attrs = Vec::new();
            if let Some(alt) = image.alt() {
                attrs.push(ImageAttr::Alt(alt.into()));
            }

            if let Smart::Custom(rendering) = image.scaling() {
                attrs.push(ImageAttr::ImageRendering(match rendering {
                    typst::visualize::ImageScaling::Pixelated => "pixelated".into(),
                    typst::visualize::ImageScaling::Smooth => "smooth".into(),
                }));
            }

            attrs
        };

        let (hash, data) = encode_image(&image);
        Image {
            data,
            format: format.into(),
            size: Axes::new(image.width() as u32, image.height() as u32),
            hash,
            attrs,
        }
    }
}

#[comemo::memoize]
fn encode_image(image: &typst::visualize::Image) -> (Fingerprint, Arc<[u8]>) {
    // steal prehash from [`typst::image::Image`]
    let hash = Fingerprint::from_u128(typst_affinite_hash(&image));

    let data = match image.kind() {
        ImageKind::Raster(raster) => match raster.format() {
            RasterFormat::Exchange(..) => raster.data().as_slice().into(),
            RasterFormat::Pixel(_) => {
                let mut buf = vec![];
                let mut encoder = PngEncoder::new(&mut buf);
                if let Some(icc_profile) = raster.icc() {
                    encoder.set_icc_profile(icc_profile.to_vec()).ok();
                }
                raster.dynamic().write_with_encoder(encoder).unwrap();
                buf.as_slice().into()
            }
        },
        ImageKind::Svg(svg) => svg.data().as_slice().into(),
    };

    (hash, data)
}
