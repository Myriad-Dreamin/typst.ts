//! OpenType glyph rendering.
//!
//! - Render glyph outlines into coverage bitmaps.
//! - Place glyphs at subpixel offsets and scale them to subpixel sizes. This is
//!   important if you plan to render more than a single glyph since inter-glyph
//!   spacing will look off if every glyph origin must be pixel-aligned.
//! - No font data structure you have to store somewhere. Just owned glyphs
//!   which you can load individually from a font, cache if you care about
//!   performance, and then render at any size.
//! - No unsafe code.
//!
//! _Note on text:_  This library does not provide any capabilities to map
//! text/characters to glyph ids. Instead, you should use a proper shaping
//! library (like [`rustybuzz`]) to do this step. This will take care of proper
//! glyph positioning, ligatures and more.
//!
//! _Note on emojis:_ This library only supports normal outlines. How to best
//! render bitmap, SVG and colored glyphs depends very much on your rendering
//! environment.
//!
//! [`rustybuzz`]: https://github.com/RazrFalcon/rustybuzz

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt::{self, Debug, Formatter};
use std::ops::{Add, Div, Mul, Sub};

use reflexo::vector::ir::Rect;
use svgtypes::SimplePathSegment;
use tiny_skia::Transform;
use wasm_bindgen::JsCast;
use web_sys::{ImageData, OffscreenCanvas, OffscreenCanvasRenderingContext2d};

use crate::device::CanvasDevice;

/// A loaded glyph that is ready for rendering.
#[derive(Debug, Clone)]
pub struct Glyph {
    /// The glyph bounding box.
    bbox: Rect,
    /// The path segments.
    segments: Vec<Segment>,
}

impl Glyph {
    /// Create a new glyph.
    pub fn new(s: &str) -> Self {
        let rect =
            reflexo_vec2bbox::Vec2BBoxPass::simple_path_bbox(s, Transform::identity()).unwrap();
        let bbox = rect.cano();
        let mut builder = Builder::default();
        for v in svgtypes::SimplifyingPathParser::from(s) {
            let v = v.unwrap();
            match v {
                SimplePathSegment::MoveTo { x, y } => builder.move_to(x as f32, y as f32),
                SimplePathSegment::LineTo { x, y } => builder.line_to(x as f32, y as f32),
                SimplePathSegment::Quadratic { x1, y1, x, y } => {
                    builder.quad_to(x1 as f32, y1 as f32, x as f32, y as f32)
                }
                SimplePathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => builder.curve_to(
                    x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
                ),
                SimplePathSegment::ClosePath => builder.close(),
            }
        }

        // bbox.lo.x.0 -= 3.0 * 128.;
        // bbox.lo.y.0 -= 3.0 * 128.;
        // bbox.hi.x.0 += 3.0 * 128.;
        // bbox.hi.y.0 += 3.0 * 128.;
        // Glyph::rasterize: left=1, top=-20, right=22, bottom=0

        Self {
            bbox,
            segments: builder.segments,
        }
    }
}

/// Builds the glyph outline.
#[derive(Default)]
struct Builder {
    segments: Vec<Segment>,
    start: Option<Point>,
    last: Point,
}

impl Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = Some(point(x, y));
        self.last = point(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.segments.push(Segment::Line(self.last, point(x, y)));
        self.last = point(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.segments
            .push(Segment::Quad(self.last, point(x1, y1), point(x2, y2)));
        self.last = point(x2, y2);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.segments.push(Segment::Cubic(
            self.last,
            point(x1, y1),
            point(x2, y2),
            point(x3, y3),
        ));
        self.last = point(x3, y3);
    }

    fn close(&mut self) {
        if let Some(start) = self.start.take() {
            self.segments.push(Segment::Line(self.last, start));
            self.last = start;
        }
    }
}

/// A path segment.
#[derive(Debug, Copy, Clone)]
enum Segment {
    /// A straight line.
    Line(Point, Point),
    /// A quadratic bezier curve.
    Quad(Point, Point, Point),
    /// A cubic bezier curve.
    Cubic(Point, Point, Point, Point),
}

impl Glyph {
    /// Rasterize the glyph.
    ///
    /// # Placing & scaling
    /// The values of `x` and `y` determine the subpixel positions at which the
    /// glyph origin should reside in some larger pixel raster (i.e. a canvas
    /// which you render text into). This is important when you're rendering the
    /// resulting bitmap into a larger pixel buffer and the glyph origin is not
    /// pixel-aligned in that raster.
    ///
    /// For example, if you want to render a glyph into your own canvas with its
    /// origin at `(3.5, 4.6)` (in pixels) you would use these exact values for
    /// `x` and `y`.
    ///
    /// The `size` defines how many pixels should correspond to `1em`
    /// horizontally and vertically. So, if you wanted to want to render your
    /// text at a size of `12px`, then `size` should be `12.0`.
    ///
    /// # Rendering into a larger canvas
    /// The result of rasterization is a coverage bitmap along with position and
    /// sizing data for it. Each individual coverage value defines how much one
    /// pixel is covered by the text. So if you have an RGB text color, you can
    /// directly use the coverage values as alpha values to form RGBA pixels.
    /// The returned `left` and `top` values define on top of which pixels in
    /// your canvas you should blend each of these new pixels.
    ///
    /// In our example, we have `glyph.rasterize(3.5, 4.6, 12.0, 12.0)`. Now,
    /// let's say the returned values are `left: 3`, `top: 1`, `width: 6` and
    /// `height: 9`. Then you need to apply the coverage values to your canvas
    /// starting at `(3, 1)` and going to `(9, 10)` row-by-row.
    pub fn rasterize(&self, x: f32, y: f32, sx: f32, sy: f32) -> Bitmap {
        // canvas.fill_with_path_2d(&Path2d::new_with_path_string(&path.d).unwrap());
        // Determine the pixel-aligned bounding box of the glyph in the larger
        // pixel raster. For y, we flip and sign and min/max because Y-up. We
        // add a bit of horizontal slack to prevent floating problems when the
        // curve is directly at the border (only needed horizontally due to
        // row-by-row data layout).
        let slack = 0.01;
        // let left = (x + s * self.bbox.x_min as f32 - slack).floor() as i32;
        // let right = (x + s * self.bbox.x_max as f32 + slack).ceil() as i32;
        // let top = (y - s * self.bbox.y_max as f32).floor() as i32;
        // let bottom = (y - s * self.bbox.y_min as f32).ceil() as i32;

        let l = x + sx * self.bbox.left().0;
        let r = x + sx * self.bbox.right().0;
        let t = y + sy * self.bbox.bottom().0;
        let b = y + sy * self.bbox.top().0;

        let left = (l.min(r) - slack).floor() as i32;
        let right = (r.max(l) + slack).ceil() as i32;
        // glyph is flipped, so we flip the top and bottom
        let top = t.min(b).floor() as i32;
        let bottom = b.max(t).ceil() as i32;

        // web_sys::console::log_1(
        //     &format!(
        //         "Glyph::rasterize: left={left}, top={top}, right={right},
        // bottom={bottom}, x={x}, y={y}, sx={sx}, sy={sy}",     )
        //     .into()
        // );

        let width = right - left;
        let height = bottom - top;

        // Create function to transform individual points.
        let dx = x - left as f32;
        let dy = y - top as f32;
        let t = |p: Point| point(dx + p.x * sx, dy + p.y * sy);

        // Draw!
        let mut canvas = Canvas::new(width as u32, height as u32);
        for &segment in &self.segments {
            match segment {
                Segment::Line(p0, p1) => canvas.line(t(p0), t(p1)),
                Segment::Quad(p0, p1, p2) => canvas.quad(t(p0), t(p1), t(p2)),
                Segment::Cubic(p0, p1, p2, p3) => canvas.cubic(t(p0), t(p1), t(p2), t(p3)),
            }
        }
        // pub(crate) fn to_html_bitmap(bitmap: &Bitmap) -> ImageData {
        //     // pub fn new_with_u8_clamped_array_and_sh(
        //     //     data: ::wasm_bindgen::Clamped<&[u8]>,
        //     //     sw: u32,
        //     //     sh: u32,
        //     // ) -> Result<ImageData, JsValue>;

        //     let w = bitmap.width

        // }

        Bitmap {
            left,
            top,
            width,
            height,
            coverage: canvas.accumulate(),
        }
    }
}

/// The result of rasterizing a glyph.
pub struct Bitmap {
    /// Horizontal pixel position (from the left) at which the bitmap should be
    /// placed in the larger raster.
    pub left: i32,
    /// Vertical pixel position (from the top) at which the bitmap should be
    /// placed in the larger raster.
    pub top: i32,
    /// The width of the coverage bitmap in pixels.
    pub width: i32,
    /// The height of the coverage bitmap in pixels.
    pub height: i32,
    /// How much each pixel should be covered, `0` means 0% coverage and `255`
    /// means 100% coverage.
    ///
    /// The length of this vector is `width * height`, with the values being
    /// stored row-by-row.
    pub coverage: ImageData,
}

impl Debug for Bitmap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Bitmap")
            .field("left", &self.left)
            .field("top", &self.top)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

// Accumulation, line and quad drawing taken from here:
// https://github.com/raphlinus/font-rs
//
// Copyright 2015 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// The internal rendering buffer.
struct Canvas {
    w: usize,
    h: usize,
    a: Vec<f32>,
}

impl Canvas {
    /// Create a completely uncovered canvas.
    fn new(w: u32, h: u32) -> Self {
        // web_sys::console::log_1(
        //     &format!(
        //         "Canvas::new: w={}, h={}, w*h={}",
        //         w,
        //         h,
        //         (w as usize) * (h as usize)
        //     )
        //     .into(),
        // );
        Self {
            w: w as usize,
            h: h as usize,
            a: vec![0.0; (w * h + 4) as usize],
        }
    }

    /// Return the accumulated coverage values.
    fn accumulate(self) -> ImageData {
        let mut acc = 0.0;
        // let clamped = self.a[..self.w * self.h]
        //     .iter()
        //     .flat_map(|c| {
        //         acc += c;
        //         let a = (255.0 * acc.abs().min(1.0)) as u8;
        //         [255, 255, 255, a]
        //     })
        //     .collect::<Box<_>>();

        let mut clamped = vec![255u8; self.w * self.h * 4];
        for (i, c) in self.a.iter().enumerate().take(self.w * self.h) {
            acc += c;
            let a = (255.0 * acc.abs().min(1.0)) as u8;
            // Method 1: Use the same alpha for all channels.
            // clamped[i * 4 + 3] = if a > 0 { 255 } else { 0 };
            // Method 2: Keep alpha.
            clamped[i * 4 + 3] = a;
            // Method 2: Keep alpha a bit.
            // clamped[i * 4 + 3] = a / 16 * 16;
        }

        ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(clamped.as_ref()),
            self.w as u32,
            self.h as u32,
        )
        .unwrap()
    }

    /// Add to a value in the accumulation buffer.
    fn add(&mut self, index: usize, delta: f32) {
        if let Some(a) = self.a.get_mut(index) {
            *a += delta;
        }
    }

    /// Draw a straight line.
    fn line(&mut self, p0: Point, p1: Point) {
        if (p0.y - p1.y).abs() <= f32::EPSILON {
            return;
        }
        let (dir, p0, p1) = if p0.y < p1.y {
            (1.0, p0, p1)
        } else {
            (-1.0, p1, p0)
        };
        let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
        let mut x = p0.x;
        let y0 = p0.y as usize;
        if p0.y < 0.0 {
            x -= p0.y * dxdy;
        }
        for y in y0..self.h.min(p1.y.ceil() as usize) {
            let linestart = y * self.w;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                self.add(linestart + x0i as usize, d - d * xmf);
                self.add(linestart + (x0i + 1) as usize, d * xmf);
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                self.add(linestart + x0i as usize, d * a0);
                if x1i == x0i + 2 {
                    self.add(linestart + (x0i + 1) as usize, d * (1.0 - a0 - am));
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.add(linestart + (x0i + 1) as usize, d * (a1 - a0));
                    for xi in x0i + 2..x1i - 1 {
                        self.add(linestart + xi as usize, d * s);
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.add(linestart + (x1i - 1) as usize, d * (1.0 - a2 - am));
                }
                self.add(linestart + x1i as usize, d * am);
            }
            x = xnext;
        }
    }

    /// Draw a quadratic bezier curve.
    fn quad(&mut self, p0: Point, p1: Point, p2: Point) {
        // How much does the curve deviate from a straight line?
        let devsq = hypot2(p0 - 2.0 * p1 + p2);

        // Check if the curve is already flat enough.
        if devsq < 0.333 {
            self.line(p0, p2);
            return;
        }

        // Estimate the required number of subdivisions for flattening.
        let tol = 3.0;
        let n = 1.0 + (tol * devsq).sqrt().sqrt().floor().min(30.0);
        let nu = n as usize;
        let step = n.recip();

        // Flatten the curve.
        let mut t = 0.0;
        let mut p = p0;
        for _ in 0..nu.saturating_sub(1) {
            t += step;

            // Evaluate the curve at `t` using De Casteljau and draw a line from
            // the last point to the new evaluated point.
            let p01 = lerp(t, p0, p1);
            let p12 = lerp(t, p1, p2);
            let pt = lerp(t, p01, p12);
            self.line(p, pt);

            // Then set the evaluated point as the start point of the new line.
            p = pt;
        }

        // Draw a final line.
        self.line(p, p2);
    }
}

// Cubic to quad conversion adapted from here:
// https://github.com/linebender/kurbo/blob/master/src/cubicbez.rs
//
// Copyright 2018 The kurbo Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

impl Canvas {
    /// Draw a cubic bezier curve.
    fn cubic(&mut self, p0: Point, p1: Point, p2: Point, p3: Point) {
        // How much does the curve deviate?
        let p1x2 = 3.0 * p1 - p0;
        let p2x2 = 3.0 * p2 - p3;
        let err = hypot2(p2x2 - p1x2);

        // Estimate the required number of subdivisions for conversion.
        let tol = 0.333;
        let max = 432.0 * tol * tol;
        let n = (err / max).powf(1.0 / 6.0).ceil().clamp(1.0, 20.0);
        let nu = n as usize;
        let step = n.recip();
        let step4 = step / 4.0;

        // Compute the derivative of the cubic.
        let dp0 = 3.0 * (p1 - p0);
        let dp1 = 3.0 * (p2 - p1);
        let dp2 = 3.0 * (p3 - p2);

        // Convert the cubics to quadratics.
        let mut t = 0.0;
        let mut p = p0;
        let mut pd = dp0;
        for _ in 0..nu {
            t += step;

            // Evaluate the curve at `t` using De Casteljau.
            let p01 = lerp(t, p0, p1);
            let p12 = lerp(t, p1, p2);
            let p23 = lerp(t, p2, p3);
            let p012 = lerp(t, p01, p12);
            let p123 = lerp(t, p12, p23);
            let pt = lerp(t, p012, p123);

            // Evaluate the derivative of the curve at `t` using De Casteljau.
            let dp01 = lerp(t, dp0, dp1);
            let dp12 = lerp(t, dp1, dp2);
            let pdt = lerp(t, dp01, dp12);

            // Determine the control point of the quadratic.
            let pc = (p + pt) / 2.0 + step4 * (pd - pdt);

            // Draw the quadratic.
            self.quad(p, pc, pt);

            p = pt;
            pd = pdt;
        }
    }
}

/// Create a point.
fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

/// Linearly interpolate between two points.
fn lerp(t: f32, p1: Point, p2: Point) -> Point {
    Point {
        x: p1.x + t * (p2.x - p1.x),
        y: p1.y + t * (p2.y - p1.y),
    }
}

/// The squared distance of the point from the origin.
fn hypot2(p: Point) -> f32 {
    p.x * p.x + p.y * p.y
}

/// A point in 2D.
#[derive(Debug, Default, Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<Point> for f32 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div<f32> for Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

pub(crate) fn blend_glyph(
    canvas: &dyn CanvasDevice,
    bitmap: &Bitmap,
    fill: &str,
    x: i32,
    y: i32,
    // sampler: S,
) -> Option<()> {
    let cw = (2 + bitmap.width) as u32;
    let ch = (2 + bitmap.height) as u32;
    let os = OffscreenCanvas::new(cw, ch).unwrap();
    let ctx = os.get_context("2d").unwrap().unwrap();
    let backed_canvas = ctx.dyn_into::<OffscreenCanvasRenderingContext2d>().unwrap();
    let ctx: &dyn CanvasDevice = &backed_canvas;

    // web_sys::console::log_1(
    //     &format!(
    //         "blend_glyph: left={}, top={}, width={}, height={} -> {x} {y}",
    //         bitmap.left, bitmap.top, bitmap.width, bitmap.height
    //     )
    //     .into(),
    // );
    // ((bitmap.height as i32 + bitmap.top) + (-(bitmap.height as f64) * 0.24) as
    // i32) as f64,
    ctx.put_image_data(&bitmap.coverage, 1., 1.);
    let gco = ctx.global_composite_operation();
    ctx.set_global_composite_operation("source-in");
    ctx.set_fill_style_str(fill);
    ctx.fill_rect(0.0, 0.0, cw as f64, ch as f64);
    ctx.set_global_composite_operation(&gco);
    // left - 1,
    // top - 1,
    // pixmap.as_ref(),
    // &sk::PixmapPaint::default(),
    // sk::Transform::identity(),
    // state.mask,

    canvas.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    canvas.draw_image_with_offscreen_canvas(
        &os,
        (x + bitmap.left - 1) as f64,
        (y + bitmap.top - 1) as f64,
    );

    Some(())
}

#[cfg(test)]
mod tests {
    use tiny_skia::Transform;

    #[test]
    fn test_glyph() {
        let rect =
        reflexo_vec2bbox::Vec2BBoxPass::simple_path_bbox("M 173 267 L 369 267 L 270 587 L 173 267 Z M 6 0 L 224 656 L 320 656 L 541 0 L 452 0 L 390 200 L 151 200 L 85 0 L 6 0 Z ", Transform::identity()).unwrap();
        let bbox = rect.cano();
        assert_eq!(bbox.left().0 as u32, 6);
        assert_eq!(bbox.top().0 as u32, 0);
        assert_eq!(bbox.right().0 as u32, 541);
        assert_eq!(bbox.bottom().0 as u32, 656);
    }
}
