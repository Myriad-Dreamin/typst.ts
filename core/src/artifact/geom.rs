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
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scalar(pub f64);

impl From<TypstScalar> for Scalar {
    fn from(typst_scalar: TypstScalar) -> Self {
        Self(typst_scalar.0)
    }
}

impl From<Scalar> for TypstScalar {
    fn from(scalar: Scalar) -> Self {
        Self(scalar.0)
    }
}

/// An absolute length.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Abs(Scalar);

impl From<TypstAbs> for Abs {
    fn from(typst_abs: TypstAbs) -> Self {
        Self(Scalar(typst_abs.to_raw()))
    }
}

impl From<Abs> for TypstAbs {
    fn from(abs: Abs) -> Self {
        Self::raw(abs.0 .0)
    }
}

/// A ratio of a whole.
///
/// _Note_: `50%` is represented as `0.5` here, but stored as `50.0` in the
/// corresponding [literal](typst::syntax::ast::Numeric).
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ratio(Scalar);

impl From<TypstRatio> for Ratio {
    fn from(typst_ratio: TypstRatio) -> Self {
        Self(Scalar(typst_ratio.get()))
    }
}

impl From<Ratio> for TypstRatio {
    fn from(ratio: Ratio) -> Self {
        Self::new(ratio.0 .0)
    }
}

/// A length that is relative to the font size.
///
/// `1em` is the same as the font size.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Em(Scalar);

impl From<TypstEm> for Em {
    fn from(typst_em: TypstEm) -> Self {
        Self(Scalar(typst_em.get()))
    }
}

impl From<Em> for TypstEm {
    fn from(em: Em) -> Self {
        Self::new(em.0 .0)
    }
}

/// A container with a horizontal and vertical component.
#[repr(C)]
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

impl<T, U> From<Axes<T>> for TypstAxes<U>
where
    T: Into<U>,
{
    fn from(axes: Axes<T>) -> Self {
        Self {
            x: axes.x.into(),
            y: axes.y.into(),
        }
    }
}

/// A point in 2D.
#[repr(C)]
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

impl From<Point> for TypstPoint {
    fn from(point: Point) -> Self {
        Self {
            x: point.x.into(),
            y: point.y.into(),
        }
    }
}

/// A scale-skew-translate transformation.
#[repr(C)]
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

impl From<Transform> for TypstTransform {
    fn from(transform: Transform) -> Self {
        Self {
            sx: transform.sx.into(),
            ky: transform.ky.into(),
            kx: transform.kx.into(),
            sy: transform.sy.into(),
            tx: transform.tx.into(),
            ty: transform.ty.into(),
        }
    }
}

/// A geometric shape with optional fill and stroke.
#[repr(C)]
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

impl From<Shape> for TypstShape {
    fn from(shape: Shape) -> Self {
        Self {
            geometry: shape.geometry.into(),
            fill: shape.fill.map(|paint| paint.into()),
            stroke: shape.stroke.map(|stroke| stroke.into()),
        }
    }
}

/// How a fill or stroke should be painted.
#[repr(C)]
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

impl From<Paint> for TypstPaint {
    fn from(paint: Paint) -> Self {
        match paint {
            Paint::Solid(color) => Self::Solid(color.into()),
        }
    }
}

/// A shape's geometry.
#[repr(C)]
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

impl From<Geometry> for TypstGeometry {
    fn from(geometry: Geometry) -> Self {
        match geometry {
            Geometry::Line(point) => Self::Line(point.into()),
            Geometry::Rect(size) => Self::Rect(size.into()),
            Geometry::Path(path) => Self::Path(path.into()),
        }
    }
}

/// A bezier path.
#[repr(C)]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Path(pub Vec<PathItem>);

impl From<TypstPath> for Path {
    fn from(typst_path: TypstPath) -> Self {
        Self(typst_path.0.into_iter().map(Into::into).collect())
    }
}

impl From<Path> for TypstPath {
    fn from(path: Path) -> Self {
        Self(path.0.into_iter().map(Into::into).collect())
    }
}

/// An item in a bezier path.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
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

impl From<PathItem> for TypstPathItem {
    fn from(path_item: PathItem) -> Self {
        match path_item {
            PathItem::MoveTo(point) => Self::MoveTo(point.into()),
            PathItem::LineTo(point) => Self::LineTo(point.into()),
            PathItem::CubicTo(point_1, point_2, point_3) => {
                Self::CubicTo(point_1.into(), point_2.into(), point_3.into())
            }
            PathItem::ClosePath => Self::ClosePath,
        }
    }
}

/// The line cap of a stroke
#[repr(C)]
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

impl From<LineCap> for TypstLineCap {
    fn from(line_cap: LineCap) -> Self {
        match line_cap {
            LineCap::Butt => Self::Butt,
            LineCap::Round => Self::Round,
            LineCap::Square => Self::Square,
        }
    }
}

/// The line join of a stroke
#[repr(C)]
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

impl From<LineJoin> for TypstLineJoin {
    fn from(line_join: LineJoin) -> Self {
        match line_join {
            LineJoin::Miter => Self::Miter,
            LineJoin::Round => Self::Round,
            LineJoin::Bevel => Self::Bevel,
        }
    }
}

/// A length, possibly expressed with contextual units.
///
/// Currently supports absolute and font-relative units, but support could quite
/// easily be extended to other units.
#[repr(C)]
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

impl From<Length> for TypstLength {
    fn from(length: Length) -> Self {
        Self {
            abs: length.abs.into(),
            em: length.em.into(),
        }
    }
}

/// The length of a dash in a line dash pattern
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum DashLength<T> {
    LineWidth,
    Length(T),
}

impl<T> From<TypstDashLength<T>> for DashLength<T> {
    fn from(typst_dash_length: TypstDashLength<T>) -> Self {
        match typst_dash_length {
            TypstDashLength::LineWidth => Self::LineWidth,
            TypstDashLength::Length(typst_length) => Self::Length(typst_length),
        }
    }
}

impl<T> From<DashLength<T>> for TypstDashLength<T> {
    fn from(dash_length: DashLength<T>) -> Self {
        match dash_length {
            DashLength::LineWidth => Self::LineWidth,
            DashLength::Length(length) => Self::Length(length),
        }
    }
}

/// A line dash pattern
#[repr(C)]
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

impl<T, DT, RT, RDT> From<DashPattern<T, DT>> for TypstDashPattern<RT, RDT>
where
    T: Into<RT>,
    DT: Into<RDT>,
{
    fn from(dash_pattern: DashPattern<T, DT>) -> Self {
        Self {
            array: dash_pattern.array.into_iter().map(|x| x.into()).collect(),
            phase: dash_pattern.phase.into(),
        }
    }
}

/// A stroke of a geometric shape.
#[repr(C)]
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

impl From<Stroke> for TypstStroke {
    fn from(stroke: Stroke) -> Self {
        Self {
            paint: stroke.paint.into(),
            thickness: stroke.thickness.into(),
            line_cap: stroke.line_cap.into(),
            line_join: stroke.line_join.into(),
            dash_pattern: stroke.dash_pattern.map(|dash_pattern| dash_pattern.into()),
            miter_limit: stroke.miter_limit.into(),
        }
    }
}

/// An 8-bit grayscale color.
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LumaColor(pub u8);

impl From<TypstLumaColor> for LumaColor {
    fn from(typst_luma_color: TypstLumaColor) -> Self {
        Self(typst_luma_color.0)
    }
}

impl From<LumaColor> for TypstLumaColor {
    fn from(luma_color: LumaColor) -> Self {
        Self::new(luma_color.0)
    }
}

/// An 8-bit RGBA color.
#[repr(C)]
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

impl From<RgbaColor> for TypstRgbaColor {
    fn from(rgba_color: RgbaColor) -> Self {
        Self::new(rgba_color.r, rgba_color.g, rgba_color.b, rgba_color.a)
    }
}

/// An 8-bit CMYK color.
#[repr(C)]
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

impl From<CmykColor> for TypstCmykColor {
    fn from(cmyk_color: CmykColor) -> Self {
        Self::new(cmyk_color.c, cmyk_color.m, cmyk_color.y, cmyk_color.k)
    }
}

/// A color in a dynamic format.
#[repr(C)]
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

impl From<Color> for TypstColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Luma(luma_color) => Self::Luma(luma_color.into()),
            Color::Rgba(rgba_color) => Self::Rgba(rgba_color.into()),
            Color::Cmyk(cmyk_color) => Self::Cmyk(cmyk_color.into()),
        }
    }
}

/// A size in 2D.
pub type Size = Axes<Abs>;
