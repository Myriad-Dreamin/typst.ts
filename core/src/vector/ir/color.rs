use super::preludes::*;

/// Item representing an 8-bit color item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct ColorItem {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorItem {
    // todo: to_css
    pub fn to_hex(self) -> String {
        let Self { r, g, b, a } = self;
        if a != 255 {
            format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
        } else {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        }
    }

    pub fn typst(&self) -> typst::visualize::Color {
        typst::visualize::Color::from_u8(self.r, self.g, self.b, self.a)
    }
}

/// A color space for mixing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ColorSpace {
    /// A perceptual color space.
    Oklab,

    /// The standard RGB color space.
    Srgb,

    /// The D65-gray color space.
    D65Gray,

    /// The linear RGB color space.
    LinearRgb,

    /// The HSL color space.
    Hsl,

    /// The HSV color space.
    Hsv,

    /// The CMYK color space.
    Cmyk,

    /// The perceptual Oklch color space.
    Oklch,
}

impl From<typst::visualize::ColorSpace> for ColorSpace {
    fn from(value: typst::visualize::ColorSpace) -> Self {
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

impl From<ColorSpace> for typst::visualize::ColorSpace {
    fn from(value: ColorSpace) -> Self {
        use typst::visualize::ColorSpace::*;
        match value {
            ColorSpace::Oklab => Oklab,
            ColorSpace::Oklch => Oklch,
            ColorSpace::Srgb => Srgb,
            ColorSpace::D65Gray => D65Gray,
            ColorSpace::LinearRgb => LinearRgb,
            ColorSpace::Hsl => Hsl,
            ColorSpace::Hsv => Hsv,
            ColorSpace::Cmyk => Cmyk,
        }
    }
}

/// Item representing an `<gradient/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct GradientItem {
    /// The path instruction.
    pub stops: Vec<(ColorItem, Scalar)>,
    /// Whether the gradient is relative to itself (its own bounding box).
    /// Otherwise, the gradient is relative to the parent bounding box.
    pub relative_to_self: Option<bool>,
    /// Whether to anti-alias the gradient (used for sharp gradients).
    pub anti_alias: bool,
    /// A color space for mixing.
    pub space: ColorSpace,
    /// The gradient kind.
    /// See [`GradientKind`] for more information.
    pub kind: GradientKind,
    /// Additional gradient styles.
    /// See [`GradientStyle`] for more information.
    pub styles: Vec<GradientStyle>,
}

/// Kind of graidents for [`GradientItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum GradientKind {
    /// Angle of a linear gradient.
    Linear(Scalar),
    /// Radius of a radial gradient.
    Radial(Scalar),
    /// Angle of a conic gradient.
    Conic(Scalar),
}

/// Attributes that is applicable to the [`GradientItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum GradientStyle {
    /// Center of a radial or conic gradient.
    Center(Point),
    /// Focal center of a radial gradient.
    FocalCenter(Point),
    /// Focal radius of a radial gradient.
    FocalRadius(Scalar),
}
