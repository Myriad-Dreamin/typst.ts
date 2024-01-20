use super::preludes::*;

/// Item representing an 8-bit color item.
///
/// It is less precise than [`Color32Item`], but it is more widely supported.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Rgba8Item {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8Item {
    /// Convert to [`typst::visualize::Color`].
    pub fn typst(&self) -> typst::visualize::Color {
        (*self).into()
    }
}

impl From<Rgba8Item> for typst::visualize::Color {
    fn from(v: Rgba8Item) -> Self {
        typst::visualize::Color::from_u8(v.r, v.g, v.b, v.a)
    }
}

/// A 32-bit color in a specific color space.
/// Note: some backends may not support 32-bit colors.
///
/// See <https://developer.chrome.com/docs/css-ui/high-definition-css-color-guide>
///
/// Detection:
///
/// ```js
/// const hasHighDynamicRange = window.matchMedia('(dynamic-range: high)').matches;
/// const hasP3Color = window.matchMedia('(color-gamut: p3)').matches;
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Color32Item {
    /// The color space.
    pub space: ColorSpace,
    /// The color value.
    pub value: [Scalar; 4],
}

/// A color space for mixing.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum ColorSpace {
    /// Luma color space.
    Luma,

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

impl ColorSpace {
    pub fn to_str(&self) -> &'static str {
        match self {
            ColorSpace::Luma => "luma",
            ColorSpace::Oklab => "oklab",
            ColorSpace::Srgb => "srgb",
            ColorSpace::D65Gray => "d65-gray",
            ColorSpace::LinearRgb => "linear-rgb",
            ColorSpace::Hsl => "hsl",
            ColorSpace::Hsv => "hsv",
            ColorSpace::Cmyk => "cmyk",
            ColorSpace::Oklch => "oklch",
        }
    }
}

impl ToString for ColorSpace {
    fn to_string(&self) -> String {
        self.to_str().to_owned()
    }
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

impl TryFrom<ColorSpace> for typst::visualize::ColorSpace {
    type Error = ();

    fn try_from(value: ColorSpace) -> Result<Self, ()> {
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

/// Item representing an `<gradient/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct GradientItem {
    /// The path instruction.
    pub stops: Vec<(Rgba8Item, Scalar)>,
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
