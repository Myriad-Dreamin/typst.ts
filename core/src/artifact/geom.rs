use serde::Deserialize;
use serde::Serialize;
pub use typst::geom::Abs as TypstAbs;
pub use typst::geom::Axes as TypstAxes;
pub use typst::geom::CmykColor as TypstCmykColor;
pub use typst::geom::Color as TypstColor;
pub use typst::geom::DashLength as TypstDashLength;
pub use typst::geom::DashPattern as TypstDashPattern;
pub use typst::geom::Em as TypstEm;
pub use typst::geom::Geometry as TypstGeometry;
pub use typst::geom::Length as TypstLength;
pub use typst::geom::LineCap as TypstLineCap;
pub use typst::geom::LineJoin as TypstLineJoin;
pub use typst::geom::LumaColor as TypstLumaColor;
pub use typst::geom::Paint as TypstPaint;
pub use typst::geom::Path as TypstPath;
pub use typst::geom::PathItem as TypstPathItem;
pub use typst::geom::Point as TypstPoint;
pub use typst::geom::Ratio as TypstRatio;
pub use typst::geom::RgbaColor as TypstRgbaColor;
pub use typst::geom::Scalar as TypstScalar;
pub use typst::geom::Shape as TypstShape;
pub use typst::geom::Stroke as TypstStroke;
pub use typst::geom::Transform as TypstTransform;

/// A 64-bit float that implements `Eq`, `Ord` and `Hash`.
///
/// Panics if it's `NaN` during any of those operations.
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scalar(pub f64);

impl From<TypstScalar> for Scalar {
    fn from(typst_scalar: TypstScalar) -> Self {
        Self(typst_scalar.0)
    }
}

impl Into<TypstScalar> for Scalar {
    fn into(self) -> TypstScalar {
        TypstScalar(self.0)
    }
}

/// An absolute length.
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Abs(Scalar);

impl From<TypstAbs> for Abs {
    fn from(typst_abs: TypstAbs) -> Self {
        Self(Scalar(typst_abs.to_raw()))
    }
}

impl Into<TypstAbs> for Abs {
    fn into(self) -> TypstAbs {
        TypstAbs::raw(self.0 .0)
    }
}

/// A ratio of a whole.
///
/// _Note_: `50%` is represented as `0.5` here, but stored as `50.0` in the
/// corresponding [literal](crate::syntax::ast::Numeric).
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ratio(Scalar);

impl From<TypstRatio> for Ratio {
    fn from(typst_ratio: TypstRatio) -> Self {
        Self(Scalar(typst_ratio.get()))
    }
}

impl Into<TypstRatio> for Ratio {
    fn into(self) -> TypstRatio {
        TypstRatio::new(self.0 .0)
    }
}

/// A length that is relative to the font size.
///
/// `1em` is the same as the font size.
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Em(Scalar);

impl From<TypstEm> for Em {
    fn from(typst_em: TypstEm) -> Self {
        Self(Scalar(typst_em.get()))
    }
}

impl Into<TypstEm> for Em {
    fn into(self) -> TypstEm {
        TypstEm::new(self.0 .0)
    }
}

/// A container with a horizontal and vertical component.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Axes<T> {
    /// The horizontal component.
    pub x: T,
    /// The vertical component.
    pub y: T,
}

impl<U, T> From<TypstAxes<U>> for Axes<T>
where
    T: From<U>,
{
    fn from(typst_axes: TypstAxes<U>) -> Self {
        Self {
            x: typst_axes.x.into(),
            y: typst_axes.y.into(),
        }
    }
}

impl<U, T> Into<TypstAxes<U>> for Axes<T>
where
    T: Into<U>,
{
    fn into(self) -> TypstAxes<U> {
        TypstAxes {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

/// A point in 2D.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    /// The x coordinate.
    pub x: Abs,
    /// The y coordinate.
    pub y: Abs,
}

impl From<TypstPoint> for Point {
    fn from(typst_point: TypstPoint) -> Self {
        Self {
            x: typst_point.x.into(),
            y: typst_point.y.into(),
        }
    }
}

impl Into<TypstPoint> for Point {
    fn into(self) -> TypstPoint {
        TypstPoint {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}

// todo: in From/To conversion flavor
/// A scale-skew-translate transformation.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub sx: Ratio,
    pub ky: Ratio,
    pub kx: Ratio,
    pub sy: Ratio,
    pub tx: Abs,
    pub ty: Abs,
}

impl From<TypstTransform> for Transform {
    fn from(typst_transform: TypstTransform) -> Self {
        Self {
            sx: typst_transform.sx.into(),
            ky: typst_transform.ky.into(),
            kx: typst_transform.kx.into(),
            sy: typst_transform.sy.into(),
            tx: typst_transform.tx.into(),
            ty: typst_transform.ty.into(),
        }
    }
}

impl Into<TypstTransform> for Transform {
    fn into(self) -> TypstTransform {
        TypstTransform {
            sx: self.sx.into(),
            ky: self.ky.into(),
            kx: self.kx.into(),
            sy: self.sy.into(),
            tx: self.tx.into(),
            ty: self.ty.into(),
        }
    }
}

/// A geometric shape with optional fill and stroke.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shape {
    /// The shape's geometry.
    pub geometry: Geometry,
    /// The shape's background fill.
    pub fill: Option<Paint>,
    /// The shape's border stroke.
    pub stroke: Option<Stroke>,
}

impl From<TypstShape> for Shape {
    fn from(typst_shape: TypstShape) -> Self {
        Self {
            geometry: typst_shape.geometry.into(),
            fill: typst_shape.fill.map(|typst_paint| typst_paint.into()),
            stroke: typst_shape.stroke.map(|typst_stroke| typst_stroke.into()),
        }
    }
}

impl Into<TypstShape> for Shape {
    fn into(self) -> TypstShape {
        TypstShape {
            geometry: self.geometry.into(),
            fill: self.fill.map(|typst_paint| typst_paint.into()),
            stroke: self.stroke.map(|typst_stroke| typst_stroke.into()),
        }
    }
}

/// How a fill or stroke should be painted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum Paint {
    /// A solid color.
    Solid(Color),
}

impl From<TypstPaint> for Paint {
    fn from(typst_paint: TypstPaint) -> Self {
        match typst_paint {
            TypstPaint::Solid(typst_color) => Self::Solid(typst_color.into()),
        }
    }
}

impl Into<TypstPaint> for Paint {
    fn into(self) -> TypstPaint {
        match self {
            Self::Solid(typst_color) => TypstPaint::Solid(typst_color.into()),
        }
    }
}

/// A shape's geometry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum Geometry {
    /// A line to a point (relative to its position).
    Line(Point),
    /// A rectangle with its origin in the topleft corner.
    Rect(Size),
    /// A bezier path.
    Path(Path),
}

impl From<TypstGeometry> for Geometry {
    fn from(typst_geometry: TypstGeometry) -> Self {
        match typst_geometry {
            TypstGeometry::Line(typst_point) => Self::Line(typst_point.into()),
            TypstGeometry::Rect(typst_size) => Self::Rect(typst_size.into()),
            TypstGeometry::Path(typst_path) => Self::Path(typst_path.into()),
        }
    }
}

impl Into<TypstGeometry> for Geometry {
    fn into(self) -> TypstGeometry {
        match self {
            Self::Line(typst_point) => TypstGeometry::Line(typst_point.into()),
            Self::Rect(typst_size) => TypstGeometry::Rect(typst_size.into()),
            Self::Path(typst_path) => TypstGeometry::Path(typst_path.into()),
        }
    }
}

/// A bezier path.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Path(pub Vec<PathItem>);

impl From<TypstPath> for Path {
    fn from(typst_path: TypstPath) -> Self {
        Self(typst_path.0.into_iter().map(Into::into).collect())
    }
}

impl Into<TypstPath> for Path {
    fn into(self) -> TypstPath {
        TypstPath(self.0.into_iter().map(Into::into).collect())
    }
}

/// An item in a bezier path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PathItem {
    MoveTo(Point),
    LineTo(Point),
    CubicTo(Point, Point, Point),
    ClosePath,
}

impl From<TypstPathItem> for PathItem {
    fn from(typst_path_item: TypstPathItem) -> Self {
        match typst_path_item {
            TypstPathItem::MoveTo(typst_point) => Self::MoveTo(typst_point.into()),
            TypstPathItem::LineTo(typst_point) => Self::LineTo(typst_point.into()),
            TypstPathItem::CubicTo(typst_point_1, typst_point_2, typst_point_3) => Self::CubicTo(
                typst_point_1.into(),
                typst_point_2.into(),
                typst_point_3.into(),
            ),
            TypstPathItem::ClosePath => Self::ClosePath,
        }
    }
}

impl Into<TypstPathItem> for PathItem {
    fn into(self) -> TypstPathItem {
        match self {
            Self::MoveTo(typst_point) => TypstPathItem::MoveTo(typst_point.into()),
            Self::LineTo(typst_point) => TypstPathItem::LineTo(typst_point.into()),
            Self::CubicTo(typst_point_1, typst_point_2, typst_point_3) => TypstPathItem::CubicTo(
                typst_point_1.into(),
                typst_point_2.into(),
                typst_point_3.into(),
            ),
            Self::ClosePath => TypstPathItem::ClosePath,
        }
    }
}

/// The line cap of a stroke
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

impl From<TypstLineCap> for LineCap {
    fn from(typst_line_cap: TypstLineCap) -> Self {
        match typst_line_cap {
            TypstLineCap::Butt => Self::Butt,
            TypstLineCap::Round => Self::Round,
            TypstLineCap::Square => Self::Square,
        }
    }
}

impl Into<TypstLineCap> for LineCap {
    fn into(self) -> TypstLineCap {
        match self {
            Self::Butt => TypstLineCap::Butt,
            Self::Round => TypstLineCap::Round,
            Self::Square => TypstLineCap::Square,
        }
    }
}

/// The line join of a stroke
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl From<TypstLineJoin> for LineJoin {
    fn from(typst_line_join: TypstLineJoin) -> Self {
        match typst_line_join {
            TypstLineJoin::Miter => Self::Miter,
            TypstLineJoin::Round => Self::Round,
            TypstLineJoin::Bevel => Self::Bevel,
        }
    }
}

impl Into<TypstLineJoin> for LineJoin {
    fn into(self) -> TypstLineJoin {
        match self {
            Self::Miter => TypstLineJoin::Miter,
            Self::Round => TypstLineJoin::Round,
            Self::Bevel => TypstLineJoin::Bevel,
        }
    }
}

/// A length, possibly expressed with contextual units.
///
/// Currently supports absolute and font-relative units, but support could quite
/// easily be extended to other units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Length {
    /// The absolute part.
    pub abs: Abs,
    /// The font-relative part.
    pub em: Em,
}

impl From<TypstLength> for Length {
    fn from(typst_length: TypstLength) -> Self {
        Self {
            abs: typst_length.abs.into(),
            em: typst_length.em.into(),
        }
    }
}

impl Into<TypstLength> for Length {
    fn into(self) -> TypstLength {
        TypstLength {
            abs: self.abs.into(),
            em: self.em.into(),
        }
    }
}

/// The length of a dash in a line dash pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DashLength<T> {
    LineWidth,
    Length(T),
}

impl<T> From<TypstDashLength<T>> for DashLength<T> {
    fn from(typst_dash_length: TypstDashLength<T>) -> Self {
        match typst_dash_length {
            TypstDashLength::LineWidth => Self::LineWidth,
            TypstDashLength::Length(typst_length) => Self::Length(typst_length.into()),
        }
    }
}

impl<T> Into<TypstDashLength<T>> for DashLength<T> {
    fn into(self) -> TypstDashLength<T> {
        match self {
            Self::LineWidth => TypstDashLength::LineWidth,
            Self::Length(typst_length) => TypstDashLength::Length(typst_length.into()),
        }
    }
}

/// A line dash pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashPattern<T, DT> {
    /// The dash array.
    pub array: Vec<DT>,
    /// The dash phase.
    pub phase: T,
}

impl<T, DT, RT, RDT> From<TypstDashPattern<T, DT>> for DashPattern<RT, RDT>
where
    T: Into<RT>,
    DT: Into<RDT>,
{
    fn from(typst_dash_pattern: TypstDashPattern<T, DT>) -> Self {
        Self {
            array: typst_dash_pattern
                .array
                .into_iter()
                .map(|x| x.into())
                .collect(),
            phase: typst_dash_pattern.phase.into(),
        }
    }
}

impl<T, DT, RT, RDT> Into<TypstDashPattern<T, DT>> for DashPattern<RT, RDT>
where
    RT: Into<T>,
    RDT: Into<DT>,
{
    fn into(self) -> TypstDashPattern<T, DT> {
        TypstDashPattern {
            array: self.array.into_iter().map(|x| x.into()).collect(),
            phase: self.phase.into(),
        }
    }
}

/// A stroke of a geometric shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stroke {
    /// The stroke's paint.
    pub paint: Paint,
    /// The stroke's thickness.
    pub thickness: Abs,
    /// The stroke's line cap.
    pub line_cap: LineCap,
    /// The stroke's line join.
    pub line_join: LineJoin,
    /// The stroke's line dash pattern.
    pub dash_pattern: Option<DashPattern<Abs, Abs>>,
    /// The miter limit. Defaults to 4.0, same as `tiny-skia`.
    pub miter_limit: Scalar,
}

impl From<TypstStroke> for Stroke {
    fn from(typst_stroke: TypstStroke) -> Self {
        Self {
            paint: typst_stroke.paint.into(),
            thickness: typst_stroke.thickness.into(),
            line_cap: typst_stroke.line_cap.into(),
            line_join: typst_stroke.line_join.into(),
            dash_pattern: typst_stroke
                .dash_pattern
                .map(|typst_dash_pattern| (typst_dash_pattern.into())),
            miter_limit: typst_stroke.miter_limit.into(),
        }
    }
}

impl Into<TypstStroke> for Stroke {
    fn into(self) -> TypstStroke {
        TypstStroke {
            paint: self.paint.into(),
            thickness: self.thickness.into(),
            line_cap: self.line_cap.into(),
            line_join: self.line_join.into(),
            dash_pattern: self
                .dash_pattern
                .map(|typst_dash_pattern| typst_dash_pattern.into()),
            miter_limit: self.miter_limit.into(),
        }
    }
}

/// An 8-bit grayscale color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LumaColor(pub u8);

impl From<TypstLumaColor> for LumaColor {
    fn from(typst_luma_color: TypstLumaColor) -> Self {
        Self(typst_luma_color.0)
    }
}

impl Into<TypstLumaColor> for LumaColor {
    fn into(self) -> TypstLumaColor {
        TypstLumaColor::new(self.0)
    }
}

/// An 8-bit RGBA color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RgbaColor {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

impl From<TypstRgbaColor> for RgbaColor {
    fn from(typst_rgba_color: TypstRgbaColor) -> Self {
        Self {
            r: typst_rgba_color.r,
            g: typst_rgba_color.g,
            b: typst_rgba_color.b,
            a: typst_rgba_color.a,
        }
    }
}

impl Into<TypstRgbaColor> for RgbaColor {
    fn into(self) -> TypstRgbaColor {
        TypstRgbaColor {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

/// An 8-bit CMYK color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CmykColor {
    /// The cyan component.
    pub c: u8,
    /// The magenta component.
    pub m: u8,
    /// The yellow component.
    pub y: u8,
    /// The key (black) component.
    pub k: u8,
}

impl From<TypstCmykColor> for CmykColor {
    fn from(typst_cmyk_color: TypstCmykColor) -> Self {
        Self {
            c: typst_cmyk_color.c,
            m: typst_cmyk_color.m,
            y: typst_cmyk_color.y,
            k: typst_cmyk_color.k,
        }
    }
}

impl Into<TypstCmykColor> for CmykColor {
    fn into(self) -> TypstCmykColor {
        TypstCmykColor {
            c: self.c,
            m: self.m,
            y: self.y,
            k: self.k,
        }
    }
}

/// A color in a dynamic format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum Color {
    /// An 8-bit luma color.
    Luma(LumaColor),
    /// An 8-bit RGBA color.
    Rgba(RgbaColor),
    /// An 8-bit CMYK color.
    Cmyk(CmykColor),
}

impl From<TypstColor> for Color {
    fn from(typst_color: TypstColor) -> Self {
        match typst_color {
            TypstColor::Luma(luma_color) => Self::Luma(luma_color.into()),
            TypstColor::Rgba(rgba_color) => Self::Rgba(rgba_color.into()),
            TypstColor::Cmyk(cmyk_color) => Self::Cmyk(cmyk_color.into()),
        }
    }
}

impl Into<TypstColor> for Color {
    fn into(self) -> TypstColor {
        match self {
            Self::Luma(luma_color) => TypstColor::Luma(luma_color.into()),
            Self::Rgba(rgba_color) => TypstColor::Rgba(rgba_color.into()),
            Self::Cmyk(cmyk_color) => TypstColor::Cmyk(cmyk_color.into()),
        }
    }
}

/// A size in 2D.
pub type Size = Axes<Abs>;
