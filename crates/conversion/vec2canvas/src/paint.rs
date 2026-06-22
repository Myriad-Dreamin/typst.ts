use std::sync::Arc;

use reflexo::{
    hash::Fingerprint,
    vector::ir::{
        self, ColorSpace, FlatGlyphItem, FontItem, GradientItem, GradientKind, GradientStyle,
        ImmutStr, Module, Rgba8Item, Scalar, Transform, VecItem,
    },
};
use tiny_skia as sk;
use typst::{
    foundations::Smart,
    visualize::{
        Color as TypstColor, ColorSpace as TypstColorSpace, ProcessColorSpace, WeightedColor,
    },
};
use web_sys::{CanvasGradient, Path2d};

use crate::{set_transform, CanvasDevice, CanvasStateGuard};

const TAU: f32 = std::f32::consts::TAU;
const CONIC_SEGMENT: usize = 360;

#[derive(Debug, Clone)]
pub enum CanvasPaint {
    Solid(ImmutStr),
    Gradient(CanvasGradientPaint),
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct CanvasGradientPaint {
    pub gradient: Arc<GradientItem>,
    pub transform: Option<Transform>,
    pub aspect_ratio: Option<f32>,
}

impl CanvasPaint {
    pub fn from_ref(module: &Module, paint: &ImmutStr) -> Self {
        if paint.starts_with("@g") {
            return resolve_gradient(module, paint).map_or(Self::Unsupported, Self::Gradient);
        }

        if paint.starts_with('@') {
            Self::Unsupported
        } else {
            Self::Solid(paint.clone())
        }
    }

    pub fn set_fill_style(&self, canvas: &dyn CanvasDevice, ts: sk::Transform) {
        match self {
            Self::Solid(color) => canvas.set_fill_style_str(color.as_ref()),
            Self::Gradient(gradient) => {
                if let Some(gradient) = gradient.to_canvas_gradient(canvas, ts) {
                    canvas.set_fill_style_canvas_gradient(&gradient);
                } else {
                    canvas.set_fill_style_str("black");
                }
            }
            Self::Unsupported => canvas.set_fill_style_str("black"),
        }
    }

    pub fn set_stroke_style(&self, canvas: &dyn CanvasDevice, ts: sk::Transform) {
        match self {
            Self::Solid(color) => canvas.set_stroke_style_str(color.as_ref()),
            Self::Gradient(gradient) => {
                if let Some(gradient) = gradient.to_canvas_gradient(canvas, ts) {
                    canvas.set_stroke_style_canvas_gradient(&gradient);
                } else {
                    canvas.set_stroke_style_str("black");
                }
            }
            Self::Unsupported => canvas.set_stroke_style_str("black"),
        }
    }

    pub fn as_solid_str(&self) -> Option<&str> {
        match self {
            Self::Solid(color) => Some(color.as_ref()),
            Self::Gradient(_) | Self::Unsupported => None,
        }
    }

    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }

    pub fn for_glyph(
        &self,
        pos: ir::Axes<Scalar>,
        glyph_scale: Scalar,
        font: &FontItem,
        glyph: u32,
    ) -> Self {
        match self {
            Self::Gradient(gradient) => {
                Self::Gradient(gradient.for_glyph(pos, glyph_scale, font, glyph))
            }
            Self::Solid(_) | Self::Unsupported => self.clone(),
        }
    }

    pub fn fill_conic_path(
        &self,
        canvas: &dyn CanvasDevice,
        ts: sk::Transform,
        clip_path: &Path2d,
        prefer_native: bool,
    ) -> bool {
        match self {
            Self::Gradient(gradient)
                if matches!(gradient.gradient.kind, GradientKind::Conic(_)) =>
            {
                gradient.fill_conic_path(canvas, ts, clip_path, prefer_native);
                true
            }
            Self::Solid(_) | Self::Gradient(_) | Self::Unsupported => false,
        }
    }

    pub fn fill_radial_path(
        &self,
        canvas: &dyn CanvasDevice,
        ts: sk::Transform,
        clip_path: &Path2d,
    ) -> bool {
        match self {
            Self::Gradient(gradient)
                if matches!(gradient.gradient.kind, GradientKind::Radial(_))
                    && gradient
                        .transform
                        .is_some_and(|transform| !can_use_native_circular_gradient(transform)) =>
            {
                gradient.fill_radial_path(canvas, ts, clip_path);
                true
            }
            Self::Solid(_) | Self::Gradient(_) | Self::Unsupported => false,
        }
    }
}

impl CanvasGradientPaint {
    fn to_canvas_gradient(
        &self,
        canvas: &dyn CanvasDevice,
        _ts: sk::Transform,
    ) -> Option<CanvasGradient> {
        let aspect_ratio = self
            .aspect_ratio
            .or_else(|| self.transform.map(transform_aspect_ratio));
        create_canvas_gradient(canvas, &self.gradient, self.transform, aspect_ratio)
    }

    fn for_glyph(
        &self,
        pos: ir::Axes<Scalar>,
        glyph_scale: Scalar,
        font: &FontItem,
        glyph: u32,
    ) -> Self {
        let aspect_ratio = self.text_aspect_ratio(font, glyph);

        let Some(transform) = self.transform.filter(|transform| !transform.is_identity()) else {
            return Self {
                gradient: self.gradient.clone(),
                transform: self.transform,
                aspect_ratio,
            };
        };

        if glyph_scale.0 == 0.0 {
            return Self {
                gradient: self.gradient.clone(),
                transform: Some(transform),
                aspect_ratio,
            };
        };

        let adjusted_x_offset = (pos.x.0 * 2.0).round();
        let adjusted_y_offset = (pos.y.0 * 2.0).round();
        let glyph_offset = Transform::from_translate(
            Scalar(-adjusted_x_offset / 2.0 * glyph_scale.0),
            Scalar(-adjusted_y_offset / 2.0 * glyph_scale.0),
        );
        let glyph_coord_scale =
            Transform::from_scale(Scalar(1.0 / glyph_scale.0), Scalar(1.0 / glyph_scale.0));
        let transform = transform
            .post_concat(Transform::from_scale(Scalar(1.0), Scalar(-1.0)))
            .post_concat(glyph_offset)
            .post_concat(glyph_coord_scale);

        Self {
            gradient: self.gradient.clone(),
            transform: Some(transform),
            aspect_ratio,
        }
    }

    fn text_aspect_ratio(&self, font: &FontItem, glyph: u32) -> Option<f32> {
        if !matches!(
            self.gradient.kind,
            GradientKind::Linear(_) | GradientKind::Conic(_)
        ) {
            return self.aspect_ratio;
        }

        self.transform
            .filter(|transform| !transform.is_identity())
            .and_then(valid_transform_aspect_ratio)
            .or_else(|| glyph_aspect_ratio(font, glyph))
            .or(self.aspect_ratio)
    }

    fn fill_conic_path(
        &self,
        canvas: &dyn CanvasDevice,
        ts: sk::Transform,
        clip_path: &Path2d,
        prefer_native: bool,
    ) {
        let GradientKind::Conic(angle) = self.gradient.kind else {
            return;
        };

        let transform = self.transform.unwrap_or_else(Transform::identity);
        let mut center = ir::Point::new(Scalar(0.5), Scalar(0.5));
        for style in &self.gradient.styles {
            if let GradientStyle::Center(c) = style {
                center = *c;
            }
        }

        if prefer_native {
            if let Some(gradient) = canvas.create_conic_gradient(
                (std::f32::consts::PI + angle.0) as f64,
                center.x.0 as f64,
                center.y.0 as f64,
            ) {
                append_color_stops(&gradient, &self.gradient);

                let _guard = CanvasStateGuard::new(canvas);
                if !set_transform(canvas, ts) {
                    return;
                }
                canvas.clip_with_path_2d(clip_path);

                let paint_transform: sk::Transform = transform.into();
                if !set_transform(canvas, ts.pre_concat(paint_transform)) {
                    return;
                }

                canvas.set_fill_style_canvas_gradient(&gradient);
                canvas.fill_rect(-4.0, -4.0, 8.0, 8.0);
                return;
            }
        }

        let aspect_ratio = gradient_aspect_ratio(
            self.aspect_ratio
                .or_else(|| self.transform.map(transform_aspect_ratio)),
        );
        let inverse_ratio = 1.0 / aspect_ratio;

        let rx = 2.0 * transform.sx.0.hypot(transform.ky.0);
        let ry = 2.0 * transform.kx.0.hypot(transform.sy.0);
        if rx == 0.0 || ry == 0.0 {
            return;
        }

        let _guard = CanvasStateGuard::new(canvas);
        if !set_transform(canvas, ts) {
            return;
        }
        canvas.clip_with_path_2d(clip_path);

        let dtheta = TAU / CONIC_SEGMENT as f32;
        let sweep = if transform_det(transform) < 0.0 { 0 } else { 1 };
        for i in 0..CONIC_SEGMENT {
            let (theta1, theta2) = conic_segment_angles(angle.0, dtheta, inverse_ratio, i);
            let center_point = (center.x.0, center.y.0);
            let segment_start = conic_segment_point(center_point, theta1);
            let segment_end = conic_segment_point(center_point, theta2);
            let (cx, cy) = transform_point(transform, center.x.0 as f64, center.y.0 as f64);
            let (x1, y1) =
                transform_point(transform, segment_start.0 as f64, segment_start.1 as f64);
            let (x2, y2) = transform_point(transform, segment_end.0 as f64, segment_end.1 as f64);

            let segment_path = format!(
                "M {cx:.6} {cy:.6} L {x1:.6} {y1:.6} A {rx:.6} {ry:.6} 0 0 {sweep} {x2:.6} {y2:.6} Z"
            );
            let Ok(segment) = Path2d::new_with_path_string(&segment_path) else {
                continue;
            };

            let t1 = i as f32 / CONIC_SEGMENT as f32;
            let t2 = (i + 1) as f32 / CONIC_SEGMENT as f32;
            let gradient = canvas.create_linear_gradient(x1, y1, x2, y2);
            add_color_stop(
                &gradient,
                0.0,
                typst_color_to_css(sample_color_stops(&self.gradient, t1)),
            );
            add_color_stop(
                &gradient,
                1.0,
                typst_color_to_css(sample_color_stops(&self.gradient, t2)),
            );
            canvas.set_fill_style_canvas_gradient(&gradient);
            canvas.fill_with_path_2d(&segment);
        }
    }

    fn fill_radial_path(&self, canvas: &dyn CanvasDevice, ts: sk::Transform, clip_path: &Path2d) {
        let GradientKind::Radial(radius) = self.gradient.kind else {
            return;
        };

        let transform = self.transform.unwrap_or_else(Transform::identity);
        let mut center = ir::Point::new(Scalar(0.5), Scalar(0.5));
        let mut focal_center = ir::Point::new(Scalar(0.5), Scalar(0.5));
        let mut focal_radius = Scalar(0.0);

        for style in &self.gradient.styles {
            match style {
                GradientStyle::Center(c) => center = *c,
                GradientStyle::FocalCenter(c) => focal_center = *c,
                GradientStyle::FocalRadius(r) => focal_radius = *r,
            }
        }

        let Some(gradient) = canvas.create_radial_gradient(
            focal_center.x.0 as f64,
            focal_center.y.0 as f64,
            focal_radius.0 as f64,
            center.x.0 as f64,
            center.y.0 as f64,
            radius.0 as f64,
        ) else {
            return;
        };
        append_color_stops(&gradient, &self.gradient);

        let _guard = CanvasStateGuard::new(canvas);
        if !set_transform(canvas, ts) {
            return;
        }
        canvas.clip_with_path_2d(clip_path);

        let paint_transform: sk::Transform = transform.into();
        if !set_transform(canvas, ts.pre_concat(paint_transform)) {
            return;
        }

        canvas.set_fill_style_canvas_gradient(&gradient);
        canvas.fill_rect(-4.0, -4.0, 8.0, 8.0);
    }
}

fn resolve_gradient(module: &Module, paint: &str) -> Option<CanvasGradientPaint> {
    let id = paint.strip_prefix("@g")?;
    let mut id = Fingerprint::try_from_str(id).ok()?;

    let transform = match module.get_item(&id)? {
        VecItem::ColorTransform(transform) => {
            id = transform.item;
            Some(transform.transform)
        }
        _ => None,
    };

    match module.get_item(&id)? {
        VecItem::Gradient(gradient) => Some(CanvasGradientPaint {
            gradient: gradient.clone(),
            transform,
            aspect_ratio: None,
        }),
        _ => None,
    }
}

fn create_canvas_gradient(
    canvas: &dyn CanvasDevice,
    gradient: &GradientItem,
    transform: Option<Transform>,
    aspect_ratio: Option<f32>,
) -> Option<CanvasGradient> {
    let transform = transform.unwrap_or_else(Transform::identity);
    let canvas_gradient = match gradient.kind {
        GradientKind::Linear(angle) => {
            let (x1, y1, x2, y2) =
                linear_gradient_points(angle.0, gradient_aspect_ratio(aspect_ratio));
            let (x1, y1, x2, y2) = transform_linear_gradient(transform, x1, y1, x2, y2);
            canvas.create_linear_gradient(x1, y1, x2, y2)
        }
        GradientKind::Radial(radius) => {
            let mut center = ir::Point::new(Scalar(0.5), Scalar(0.5));
            let mut focal_center = ir::Point::new(Scalar(0.5), Scalar(0.5));
            let mut focal_radius = Scalar(0.0);

            for style in &gradient.styles {
                match style {
                    GradientStyle::Center(c) => center = *c,
                    GradientStyle::FocalCenter(c) => focal_center = *c,
                    GradientStyle::FocalRadius(r) => focal_radius = *r,
                }
            }

            let (fx, fy) =
                transform_point(transform, focal_center.x.0 as f64, focal_center.y.0 as f64);
            let (cx, cy) = transform_point(transform, center.x.0 as f64, center.y.0 as f64);
            let radius_scale = transform_radius_scale(transform);
            canvas.create_radial_gradient(
                fx,
                fy,
                focal_radius.0 as f64 * radius_scale,
                cx,
                cy,
                radius.0 as f64 * radius_scale,
            )?
        }
        GradientKind::Conic(angle) => {
            let mut center = ir::Point::new(Scalar(0.5), Scalar(0.5));
            for style in &gradient.styles {
                if let GradientStyle::Center(c) = style {
                    center = *c;
                }
            }

            let (cx, cy) = transform_point(transform, center.x.0 as f64, center.y.0 as f64);
            canvas.create_conic_gradient((std::f32::consts::PI + angle.0) as f64, cx, cy)?
        }
    };

    append_color_stops(&canvas_gradient, gradient);
    Some(canvas_gradient)
}

fn append_color_stops(canvas_gradient: &CanvasGradient, gradient: &GradientItem) {
    for window in gradient.stops.windows(2) {
        let (start_c, start_t) = window[0];
        let (end_c, end_t) = window[1];

        add_color_stop(canvas_gradient, start_t.0, rgba_to_css(start_c));

        let len = if gradient.anti_alias {
            (256 / gradient.stops.len() as u32).max(2)
        } else {
            2
        };

        for i in 1..(len - 1) {
            let t0 = i as f32 / (len - 1) as f32;
            let t = start_t.0 + (end_t.0 - start_t.0) * t0;
            add_color_stop(
                canvas_gradient,
                t,
                typst_color_to_css(sample_color_stops(gradient, t)),
            );
        }

        add_color_stop(canvas_gradient, end_t.0, rgba_to_css(end_c));
    }
}

fn add_color_stop(canvas_gradient: &CanvasGradient, offset: f32, color: String) {
    let _ = canvas_gradient.add_color_stop(offset.clamp(0.0, 1.0), &color);
}

fn sample_color_stops(gradient: &GradientItem, t: f32) -> TypstColor {
    let t = t.clamp(0.0, 1.0);
    let stops = &gradient.stops;
    let mut j = stops.partition_point(|(_, ratio)| ratio.0 < t);

    if j == 0 {
        while stops.get(j + 1).is_some_and(|(_, r)| r.0 == 0.0) {
            j += 1;
        }

        return rgba_to_typst(stops[j].0);
    }

    if j >= stops.len() {
        return rgba_to_typst(stops[stops.len() - 1].0);
    }

    let (col_0, pos_0) = stops[j - 1];
    let (col_1, pos_1) = stops[j];
    let t = (t - pos_0.0) / (pos_1.0 - pos_0.0);
    let col_0 = rgba_to_typst(col_0);
    let col_1 = rgba_to_typst(col_1);

    let Some(mixing_space) = color_space_to_typst(gradient.space) else {
        return col_0;
    };

    TypstColor::mix_iter(
        [
            WeightedColor::new(col_0.clone(), (1.0 - t) as f64),
            WeightedColor::new(col_1, t as f64),
        ],
        Smart::Custom(mixing_space),
    )
    .unwrap_or(col_0)
}

fn rgba_to_typst(color: Rgba8Item) -> TypstColor {
    TypstColor::from_u8(color.r, color.g, color.b, color.a)
}

fn color_space_to_typst(space: ColorSpace) -> Option<TypstColorSpace> {
    let process = match space {
        ColorSpace::Luma => return None,
        ColorSpace::Oklab => ProcessColorSpace::Oklab,
        ColorSpace::Srgb => ProcessColorSpace::Srgb,
        ColorSpace::D65Gray => ProcessColorSpace::D65Gray,
        ColorSpace::LinearRgb => ProcessColorSpace::LinearRgb,
        ColorSpace::Hsl => ProcessColorSpace::Hsl,
        ColorSpace::Hsv => ProcessColorSpace::Hsv,
        ColorSpace::Cmyk => ProcessColorSpace::Cmyk,
        ColorSpace::Oklch => ProcessColorSpace::Oklch,
    };
    Some(TypstColorSpace::Process(process))
}

fn typst_color_to_css(color: TypstColor) -> String {
    let (r, g, b, a) = color.to_rgb().into_format::<u8, u8>().into_components();
    rgba_to_css(Rgba8Item { r, g, b, a })
}

fn rgba_to_css(color: Rgba8Item) -> String {
    let Rgba8Item { r, g, b, a } = color;
    if a == 255 {
        let shorter = format!("#{r:02x}{g:02x}{b:02x}");
        if shorter.chars().nth(1) == shorter.chars().nth(2)
            && shorter.chars().nth(3) == shorter.chars().nth(4)
            && shorter.chars().nth(5) == shorter.chars().nth(6)
        {
            return format!(
                "#{}{}{}",
                shorter.chars().nth(1).unwrap(),
                shorter.chars().nth(3).unwrap(),
                shorter.chars().nth(5).unwrap()
            );
        }
        return shorter;
    }

    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
}

fn transform_aspect_ratio(transform: Transform) -> f32 {
    valid_transform_aspect_ratio(transform).unwrap_or(1.0)
}

fn valid_transform_aspect_ratio(transform: Transform) -> Option<f32> {
    let width = transform.sx.0.hypot(transform.ky.0);
    let height = transform.kx.0.hypot(transform.sy.0);

    if width.is_finite() && height.is_finite() && height != 0.0 {
        Some(width / height)
    } else {
        None
    }
}

fn glyph_aspect_ratio(font: &FontItem, glyph: u32) -> Option<f32> {
    let (width, height) = match font.get_glyph(glyph)?.as_ref() {
        FlatGlyphItem::Outline(outline) => {
            let mut path = convert_path(&outline.d)?;
            if let Some(transform) = &outline.ts {
                let transform: tiny_skia_path::Transform = (**transform).into();
                path = path.transform(transform)?;
            }

            let bounds = path.bounds();
            (bounds.width(), bounds.height())
        }
        FlatGlyphItem::Image(image) => (image.image.size.x.0, image.image.size.y.0),
        FlatGlyphItem::None => return None,
    };

    if width.is_finite() && height.is_finite() && width != 0.0 && height != 0.0 {
        Some((width / height).abs())
    } else {
        None
    }
}

fn convert_path(path_data: &str) -> Option<tiny_skia_path::Path> {
    let mut builder = tiny_skia_path::PathBuilder::new();

    for segment in svgtypes::SimplifyingPathParser::from(path_data) {
        let segment = segment.ok()?;

        match segment {
            svgtypes::SimplePathSegment::MoveTo { x, y } => builder.move_to(x as f32, y as f32),
            svgtypes::SimplePathSegment::LineTo { x, y } => builder.line_to(x as f32, y as f32),
            svgtypes::SimplePathSegment::Quadratic { x1, y1, x, y } => {
                builder.quad_to(x1 as f32, y1 as f32, x as f32, y as f32)
            }
            svgtypes::SimplePathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => builder.cubic_to(
                x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
            ),
            svgtypes::SimplePathSegment::ClosePath => builder.close(),
        }
    }

    builder.finish()
}

fn transform_point(transform: Transform, x: f64, y: f64) -> (f64, f64) {
    (
        f64::from(transform.sx.0) * x + f64::from(transform.kx.0) * y + f64::from(transform.tx.0),
        f64::from(transform.ky.0) * x + f64::from(transform.sy.0) * y + f64::from(transform.ty.0),
    )
}

fn transform_det(transform: Transform) -> f32 {
    transform.sx.0 * transform.sy.0 - transform.kx.0 * transform.ky.0
}

fn transform_linear_gradient(
    transform: Transform,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> (f64, f64, f64, f64) {
    let (sx, ky, kx, sy) = (
        f64::from(transform.sx.0),
        f64::from(transform.ky.0),
        f64::from(transform.kx.0),
        f64::from(transform.sy.0),
    );
    let det = sx * sy - kx * ky;
    if det.abs() <= f64::EPSILON {
        let (x1, y1) = transform_point(transform, x1, y1);
        let (x2, y2) = transform_point(transform, x2, y2);
        return (x1, y1, x2, y2);
    }

    let vx = x2 - x1;
    let vy = y2 - y1;
    let ix = (sy * vx - ky * vy) / det;
    let iy = (-kx * vx + sx * vy) / det;
    let src_len2 = vx * vx + vy * vy;
    let dst_len2 = ix * ix + iy * iy;
    if src_len2 <= f64::EPSILON || dst_len2 <= f64::EPSILON {
        let (x1, y1) = transform_point(transform, x1, y1);
        let (x2, y2) = transform_point(transform, x2, y2);
        return (x1, y1, x2, y2);
    }

    let (x1, y1) = transform_point(transform, x1, y1);
    let scale = src_len2 / dst_len2;
    (x1, y1, x1 + ix * scale, y1 + iy * scale)
}

fn transform_radius_scale(transform: Transform) -> f64 {
    let x = transform.sx.0.hypot(transform.ky.0);
    let y = transform.kx.0.hypot(transform.sy.0);
    f64::from(x.max(y)).max(0.0)
}

fn can_use_native_circular_gradient(transform: Transform) -> bool {
    const EPS: f32 = 1e-4;

    transform.sx.0.is_finite()
        && transform.sy.0.is_finite()
        && transform.sx.0 > 0.0
        && transform.sy.0 > 0.0
        && transform.kx.0.abs() < EPS
        && transform.ky.0.abs() < EPS
        && (transform.sx.0 - transform.sy.0).abs() <= EPS * transform.sx.0.max(transform.sy.0)
}

fn gradient_aspect_ratio(aspect_ratio: Option<f32>) -> f32 {
    aspect_ratio
        .filter(|ratio| ratio.is_finite() && *ratio != 0.0)
        .unwrap_or(1.0)
}

fn correct_aspect_ratio(angle: f32, aspect_ratio: f32) -> f32 {
    (angle.sin() / aspect_ratio.abs())
        .atan2(angle.cos())
        .rem_euclid(TAU)
}

fn conic_segment_angles(angle: f32, dtheta: f32, inverse_ratio: f32, i: usize) -> (f32, f32) {
    (
        -correct_aspect_ratio(angle + dtheta * i as f32, inverse_ratio),
        -correct_aspect_ratio(angle + dtheta * (i + 1) as f32, inverse_ratio),
    )
}

fn conic_segment_point(center: (f32, f32), theta: f32) -> (f32, f32) {
    (-2.0 * theta.cos() + center.0, 2.0 * theta.sin() + center.1)
}

fn linear_gradient_points(angle: f32, aspect_ratio: f32) -> (f64, f64, f64, f64) {
    let angle = correct_aspect_ratio(angle, aspect_ratio);
    let sin = angle.sin();
    let cos = angle.cos();
    let length = sin.abs() + cos.abs();
    let (x1, y1) = match angle {
        angle if angle < std::f32::consts::FRAC_PI_2 => (0.0, 0.0),
        angle if angle < std::f32::consts::PI => (1.0, 0.0),
        angle if angle < std::f32::consts::PI * 1.5 => (1.0, 1.0),
        _ => (0.0, 1.0),
    };

    (
        x1,
        y1,
        x1 + f64::from(cos * length),
        y1 + f64::from(sin * length),
    )
}
