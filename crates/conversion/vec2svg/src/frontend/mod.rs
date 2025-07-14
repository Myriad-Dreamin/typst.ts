pub(crate) mod context;
pub(crate) mod dynamic_layout;
pub(crate) mod flat;
pub(crate) mod incremental;

pub use dynamic_layout::DynamicLayoutSvgExporter;
pub use incremental::{IncrSvgDocClient, IncrSvgDocServer, IncrementalRenderContext};

use std::{collections::HashSet, f32::consts::TAU, fmt::Write, sync::Arc};

use reflexo::hash::{item_hash128, Fingerprint, FingerprintBuilder};
use reflexo_typst2vec::{
    ir::{
        self, Axes, FlatGlyphItem, GlyphRef, GradientItem, GradientKind, GradientStyle, Module,
        Page, Scalar, Size, VecItem,
    },
    utils::ToCssExt,
    IntoTypst, TryIntoTypst,
};
use typst::{
    layout::{Angle, Quadrant},
    visualize::{Color, ColorSpace, Hsl, Hsv, WeightedColor},
};

use crate::{
    backend::{SvgGlyphBuilder, SvgText, SvgTextNode},
    ExportFeature, SvgDataSelection,
};
use context::{PaintFillMap, RenderContext, StyleDefMap};

pub struct SvgExporter<Feat: ExportFeature> {
    pub _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: ExportFeature> Default for SvgExporter<Feat> {
    fn default() -> Self {
        Self {
            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> SvgExporter<Feat> {
    /// Get header by pages.
    pub(crate) fn header(output: &[Page]) -> String {
        // calculate the width and height of the svg
        let w = output
            .iter()
            .map(|p| p.size.x.0.ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or_default();
        let h = output.iter().map(|p| p.size.y.0.ceil()).sum::<f32>();

        Self::header_inner(w, h)
    }

    /// Render the header of SVG.
    /// <svg> .. </svg>
    /// ^^^^^
    fn header_inner(w: f32, h: f32) -> String {
        format!(
            r#"<svg style="overflow: visible;" class="typst-doc" viewBox="0 0 {w:.3} {h:.3}" width="{w:.3}" height="{h:.3}" data-width="{w:.3}" data-height="{h:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
        )
    }

    /// Render the style for SVG
    /// <svg> <style/> .. </svg>
    ///       ^^^^^^^^
    /// See `StyleDefMap`.
    pub fn style_defs(style_defs: StyleDefMap, svg: &mut Vec<SvgText>) {
        // style defs
        svg.push(r#"<style type="text/css">"#.into());

        // sort and push the style defs
        let mut style_defs = style_defs.into_iter().collect::<Vec<_>>();
        style_defs.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(style_defs.into_iter().map(|v| SvgText::Plain(v.1)));

        svg.push("</style>".into());
    }

    /// Render the gradients for SVG
    /// <svg> <defs> <gradient/> </defs> .. </svg>
    ///              ^^^^^^^^^^^
    pub fn gradients<'a>(
        gradients: impl Iterator<Item = (&'a Fingerprint, &'a GradientItem)>,
        svg: &mut Vec<SvgText>,
    ) {
        let mut sub_gradients = HashSet::<(Fingerprint, SVGSubGradient)>::default();

        // todo: aspect ratio
        for (id, gradient) in gradients {
            match &gradient.kind {
                GradientKind::Linear(angle) => {
                    // todo: use native angle
                    let angle = typst::layout::Angle::rad(angle.0 as f64);

                    // todo: correct aspect ratio
                    // let angle = Gradient::correct_aspect_ratio(linear.angle, *ratio);
                    let (sin, cos) = (angle.sin(), angle.cos());
                    let length = sin.abs() + cos.abs();
                    let (x1, y1, x2, y2) = match angle.quadrant() {
                        Quadrant::First => (0.0, 0.0, cos * length, sin * length),
                        Quadrant::Second => (1.0, 0.0, cos * length + 1.0, sin * length),
                        Quadrant::Third => (1.0, 1.0, cos * length + 1.0, sin * length + 1.0),
                        Quadrant::Fourth => (0.0, 1.0, cos * length, sin * length + 1.0),
                    };

                    svg.push(SvgText::Plain(
                        format!(
                            r##"<linearGradient id="{}" spreadMethod="pad" gradientUnits="userSpaceOnUse" x1="{:.3}" y1="{:.3}" x2="{:.3}" y2="{:.3}">"##,
                            id.as_svg_id("g"),
                            x1, y1, x2, y2,
                        )
                    ))
                }
                GradientKind::Radial(radius) => {
                    let mut center = &Axes::new(Scalar(0.5), Scalar(0.5));
                    let mut focal_center = &Axes::new(Scalar(0.5), Scalar(0.5));
                    let mut focal_radius = &Scalar(0.);
                    for s in &gradient.styles {
                        match s {
                            GradientStyle::Center(c) => {
                                center = c;
                            }
                            GradientStyle::FocalCenter(c) => {
                                focal_center = c;
                            }
                            GradientStyle::FocalRadius(r) => {
                                focal_radius = r;
                            }
                        }
                    }

                    svg.push(SvgText::Plain(
                        format!(
                            r##"<radialGradient id="{}" spreadMethod="pad" gradientUnits="userSpaceOnUse" cx="{:.3}" cy="{:.3}" r="{:.3}" fx="{:.3}" fy="{:.3}" fr="{:.3}">"##,
                            id.as_svg_id("g"),
                            center.x.0, center.y.0, radius.0, focal_center.x.0, focal_center.y.0, focal_radius.0,
                        )
                    ));
                }
                GradientKind::Conic(angle) => {
                    svg.push(SvgText::Plain(
                        format!(
                            r##"<pattern id="{}" viewBox="0 0 1 1" preserveAspectRatio="none" patternUnits="userSpaceOnUse" width="2" height="2" x="-0.5" y="-0.5">"##,
                            id.as_svg_id("g"),
                        )
                    ));

                    // The rotation angle, negated to match rotation in PNG.
                    // todo: use native angle
                    // let angle = Gradient::correct_aspect_ratio(angle, *ratio);
                    // let angle = typst::geom::Angle::rad(angle.0 as f64);
                    let angle: f32 = -(angle.0).rem_euclid(TAU);
                    let mut center = &Axes::new(Scalar(0.5), Scalar(0.5));
                    for s in &gradient.styles {
                        if let GradientStyle::Center(c) = s {
                            center = c;
                        }
                    }

                    // We build an arg segment for each segment of a circle.
                    let dtheta = TAU / CONIC_SEGMENT as f32;
                    for i in 0..CONIC_SEGMENT {
                        let theta1 = dtheta * i as f32;
                        let theta2 = dtheta * (i + 1) as f32;

                        // Create the path for the segment.
                        let mut builder = SvgPath2DBuilder::default();
                        builder.move_to(
                            correct_pattern_pos(center.x.0),
                            correct_pattern_pos(center.y.0),
                        );
                        builder.line_to(
                            correct_pattern_pos(-2.0 * (theta1 + angle).cos() + center.x.0),
                            correct_pattern_pos(2.0 * (theta1 + angle).sin() + center.y.0),
                        );
                        builder.arc(
                            (2.0, 2.0),
                            0.0,
                            0,
                            1,
                            (
                                correct_pattern_pos(-2.0 * (theta2 + angle).cos() + center.x.0),
                                correct_pattern_pos(2.0 * (theta2 + angle).sin() + center.y.0),
                            ),
                        );
                        builder.close();

                        let t1 = (i as f32) / CONIC_SEGMENT as f32;
                        let t2 = (i + 1) as f32 / CONIC_SEGMENT as f32;
                        let subgradient = SVGSubGradient {
                            center: *center,
                            t0: Angle::rad((theta1 + angle) as f64),
                            t1: Angle::rad((theta2 + angle) as f64),
                            c0: sample_color_stops(gradient, t1),
                            c1: sample_color_stops(gradient, t2),
                        };
                        let f = Fingerprint::from_u128(item_hash128(&subgradient));
                        sub_gradients.insert((f, subgradient));

                        svg.push(SvgText::Plain(format!(
                            r##"<path d="{}" fill="url(#{})" stroke="url(#{})" stroke-width="0" shape-rendering="optimizeSpeed"/>"##,
                            builder.0,
                            f.as_svg_id("g"),
                            f.as_svg_id("g"),
                        )));
                    }

                    svg.push(SvgText::Plain("</pattern>".to_owned()));

                    // We skip the default stop generation code.
                    continue;
                }
            }

            for window in gradient.stops.windows(2) {
                let (start_c, start_t) = &window[0];
                let (end_c, end_t) = &window[1];

                svg.push(SvgText::Plain(format!(
                    r##"<stop offset="{}" stop-color="{}"/>"##,
                    RatioRepr(start_t.0),
                    start_c.to_css(),
                )));

                // Generate (256 / len) stops between the two stops.
                // This is a workaround for a bug in many readers:
                // They tend to just ignore the color space of the gradient.
                // The goal is to have smooth gradients but not to balloon the file size
                // too much if there are already a lot of stops as in most presets.
                let len = if gradient.anti_alias {
                    (256 / gradient.stops.len() as u32).max(2)
                } else {
                    2
                };

                for i in 1..(len - 1) {
                    let t0 = i as f32 / (len - 1) as f32;
                    let t = start_t.0 + (end_t.0 - start_t.0) * t0;
                    let c = sample_color_stops(gradient, t);

                    svg.push(SvgText::Plain(format!(
                        r##"<stop offset="{}" stop-color="{}"/>"##,
                        RatioRepr(t),
                        c.to_css(),
                    )));
                }

                svg.push(SvgText::Plain(format!(
                    r##"<stop offset="{}" stop-color="{}"/>"##,
                    RatioRepr(end_t.0),
                    end_c.to_css(),
                )));
            }

            svg.push(SvgText::Plain(match gradient.kind {
                GradientKind::Linear(..) => "</linearGradient>".to_owned(),
                GradientKind::Radial(..) => "</radialGradient>".to_owned(),
                GradientKind::Conic(..) => "</pattern>".to_owned(),
            }));
        }

        for (id, gradient) in sub_gradients {
            let x1 = 2.0 - gradient.t0.cos() as f32 + gradient.center.x.0;
            let y1 = gradient.t0.sin() as f32 + gradient.center.y.0;
            let x2 = 2.0 - gradient.t1.cos() as f32 + gradient.center.x.0;
            let y2 = gradient.t1.sin() as f32 + gradient.center.y.0;

            svg.push(SvgText::Plain(format!(
                r##"<linearGradient id="{}"  gradientUnits="objectBoundingBox" x1="{:.3}" y1="{:.3}" x2="{:.3}" y2="{:.3}">"##,
                id.as_svg_id("g"),
                x1, y1, x2, y2,
            )));

            svg.push(SvgText::Plain(format!(
                r##"<stop offset="0" stop-color="{}"/>"##,
                gradient.c0.to_css(),
            )));

            svg.push(SvgText::Plain(format!(
                r##"<stop offset="1" stop-color="{}"/>"##,
                gradient.c1.to_css(),
            )));

            svg.push(SvgText::Plain("</linearGradient>".to_owned()));
        }
    }

    pub fn patterns(
        patterns: impl Iterator<Item = (Fingerprint, Size, Arc<SvgTextNode>)>,
        svg: &mut Vec<SvgText>,
    ) {
        for (id, size, pattern) in patterns {
            svg.push(SvgText::Plain(format!(
                r##"<pattern id="{}" patternUnits="userSpaceOnUse" width="{:.3}" height="{:.3}" viewBox="0 0 {:.3} {:.3}">"##,
                id.as_svg_id("g"),
                size.x.0, size.y.0,
                size.x.0, size.y.0,
            )));

            svg.push(SvgText::Content(pattern));

            svg.push(SvgText::Plain("</pattern>".to_owned()));
        }
    }

    /// Render pages into the entire SVG
    pub fn render(
        module: &Module,
        pages: &[Page],
        parts: Option<SvgDataSelection>,
    ) -> Vec<SvgText> {
        if !module.glyphs.is_empty() {
            panic!("Glyphs should be loaded before rendering.");
        }

        let mut t = SvgTask::<Feat>::default();
        let mut svg_body = vec![];
        t.render(module, pages, &mut svg_body);
        let patterns = t.render_patterns(module);

        // note in order!: pattern may use glyphs
        let glyphs = t.render_glyphs(module.glyphs_all());

        let gradients = t
            .gradients
            .iter()
            .filter_map(|id| match module.get_item(id) {
                Some(VecItem::Gradient(g)) => Some((id, g.as_ref())),
                _ => {
                    // #[cfg(debug_assertions)]
                    panic!("Invalid gradient reference: {}", id.as_svg_id("g"));
                    #[allow(unreachable_code)]
                    None
                }
            });

        let parts = parts.as_ref();
        let with_css = parts.is_none_or(|parts| parts.css);
        let with_defs = parts.is_none_or(|parts| parts.defs);
        let with_body = parts.is_none_or(|parts| parts.body);
        let with_js = parts.is_none_or(|parts| parts.js);

        let mut svg = vec![
            SvgText::Plain(Self::header(pages)),
            // base style
        ];

        if Feat::WITH_BUILTIN_CSS && with_css {
            svg.push(r#"<style type="text/css">"#.into());
            svg.push(include_str!("./typst.svg.css").into());
            svg.push("</style>".into());
        }

        if with_defs {
            // attach the glyph defs, clip paths, and style defs
            svg.push(r#"<defs class="glyph">"#.into());
            svg.extend(glyphs);
            svg.push("</defs>".into());
            svg.push(r#"<defs class="clip-path">"#.into());
            Self::gradients(gradients, &mut svg);
            Self::patterns(patterns.into_iter(), &mut svg);
            svg.push("</defs>".into());
            Self::style_defs(t.style_defs, &mut svg);
        }

        // body
        if with_body {
            svg.append(&mut svg_body);
        }

        if Feat::WITH_RESPONSIVE_JS && with_js {
            // attach the javascript for animations
            svg.push(r#"<script type="text/javascript">"#.into());
            svg.push(include_str!("./typst.svg.js").into());
            svg.push("</script>".into());
        }

        // close SVG
        svg.push("</svg>".into());

        svg
    }
}

/// The task context for exporting svg.
/// It is also as a namespace for all the functions used in the task.
pub struct SvgTask<'a, Feat: ExportFeature> {
    /// A fingerprint builder for generating unique id.
    pub fingerprint_builder: FingerprintBuilder,

    /// Stores the style definitions used in the document.
    pub style_defs: StyleDefMap,
    /// Stores the gradient used in the document.
    pub gradients: PaintFillMap,
    /// Stores the patterns used in the document.
    pub patterns: PaintFillMap,

    _feat_phantom: std::marker::PhantomData<&'a Feat>,
}

/// Unfortunately, `Default` derive does not work for generic structs.
impl<Feat: ExportFeature> Default for SvgTask<'_, Feat> {
    fn default() -> Self {
        Self {
            fingerprint_builder: FingerprintBuilder::default(),

            style_defs: StyleDefMap::default(),
            gradients: PaintFillMap::default(),
            patterns: PaintFillMap::default(),

            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> SvgTask<'_, Feat> {
    /// Return integral page size for showing document.
    pub(crate) fn page_size(sz: Size) -> Axes<u32> {
        let (width_px, height_px) = {
            let width_px = (sz.x.0.ceil()).round().max(1.0) as u32;
            let height_px = (sz.y.0.ceil()).round().max(1.0) as u32;

            (width_px, height_px)
        };

        Axes::new(width_px, height_px)
    }

    /// fork a render task with module.
    pub fn get_render_context<'m, 't>(
        &'t mut self,
        module: &'m ir::Module,
    ) -> RenderContext<'m, 't, Feat> {
        RenderContext::<Feat> {
            module,

            fingerprint_builder: &mut self.fingerprint_builder,

            _style_defs: &mut self.style_defs,
            gradients: &mut self.gradients,
            patterns: &mut self.patterns,

            should_attach_debug_info: Feat::SHOULD_ATTACH_DEBUG_INFO,
            should_render_text_element: true,
            use_stable_glyph_id: true,
            should_rasterize_text: true,

            _feat_phantom: Default::default(),
        }
    }

    /// Render glyphs into the svg_body.
    pub fn render_glyphs<'a, I: Iterator<Item = (GlyphRef, &'a FlatGlyphItem)>>(
        &mut self,
        glyphs: I,
    ) -> Vec<SvgText> {
        let mut render_task = SvgGlyphBuilder {};

        let mut svg_body = Vec::new();

        for (abs_ref, item) in glyphs {
            svg_body.push(SvgText::Plain(
                render_task
                    .render_glyph(&abs_ref.as_svg_id("g"), item)
                    .unwrap_or_default(),
            ))
        }

        svg_body
    }

    pub fn collect_patterns(
        &mut self,
        render: impl Fn(&mut Self, &Fingerprint) -> Option<(Fingerprint, Size, Arc<SvgTextNode>)>,
    ) -> Vec<(Fingerprint, Size, Arc<SvgTextNode>)> {
        let mut used = std::mem::take(&mut self.patterns);
        let mut patterns = vec![];

        patterns.extend(used.iter().filter_map(|id| render(self, id)));
        if self.patterns.is_empty() {
            return patterns;
        }

        loop {
            let mut updated = false;
            for k in std::mem::take(&mut self.patterns) {
                if used.insert(k) {
                    if let Some(res) = render(self, &k) {
                        patterns.push(res);
                    }
                    updated = true;
                }
            }

            if !updated {
                break;
            }
        }

        patterns
    }
}

/// Maps a coordinate in a unit size square to a coordinate in the pattern.
fn correct_pattern_pos(x: f32) -> f32 {
    (x + 0.5) / 2.0
}

#[derive(Default)]
struct SvgPath2DBuilder(pub String);

/// See: https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths
impl SvgPath2DBuilder {
    #[allow(dead_code)]
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        write!(
            &mut self.0,
            "M {} {} H {} V {} H {} Z",
            x,
            y,
            x + w,
            y + h,
            x
        )
        .unwrap();
    }
}

impl SvgPath2DBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "M {x} {y} ").unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "L {x} {y} ").unwrap();
    }

    /// Creates an arc path.
    fn arc(
        &mut self,
        radius: (f32, f32),
        x_axis_rot: f32,
        large_arc_flag: u32,
        sweep_flag: u32,
        pos: (f32, f32),
    ) {
        write!(
            &mut self.0,
            "A {rx} {ry} {x_axis_rot} {large_arc_flag} {sweep_flag} {x} {y} ",
            rx = radius.0,
            ry = radius.1,
            x = pos.0,
            y = pos.1,
        )
        .unwrap();
    }

    fn close(&mut self) {
        write!(&mut self.0, "Z ").unwrap();
    }
}

/// The number of segments in a conic gradient.
/// This is a heuristic value that seems to work well.
/// Smaller values could be interesting for optimization.
const CONIC_SEGMENT: usize = 360;

/// A subgradient for conic gradients.
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
struct SVGSubGradient {
    /// The center point of the gradient.
    center: Axes<Scalar>,
    /// The start point of the subgradient.
    t0: Angle,
    /// The end point of the subgradient.
    t1: Angle,
    /// The color at the start point of the subgradient.
    c0: Color,
    /// The color at the end point of the subgradient.
    c1: Color,
}

/// Sample the stops at a given position.
// todo: use native approach
fn sample_color_stops(gradient: &GradientItem, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mut low = 0;
    let mut high = gradient.stops.len();

    let mixing_space = gradient.space.try_into_typst().unwrap();
    let stops = &gradient.stops;

    while low < high {
        let mid = (low + high) / 2;
        if stops[mid].1 .0 < t {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    if low == 0 {
        low = 1;
    }
    let (col_0, pos_0) = &stops[low - 1];
    let (col_1, pos_1) = &stops[low];
    let t = (t - pos_0.0) / (pos_1.0 - pos_0.0);
    let col_0 = (*col_0).into_typst();
    let col_1 = (*col_1).into_typst();

    let out = Color::mix_iter(
        [
            WeightedColor::new(col_0, (1.0 - t) as f64),
            WeightedColor::new(col_1, t as f64),
        ],
        mixing_space,
    )
    .unwrap();

    // Special case for handling multi-turn hue interpolation.
    if mixing_space == ColorSpace::Hsl || mixing_space == ColorSpace::Hsv {
        let color_0 = col_0.to_space(mixing_space);
        let color_1 = col_1.to_space(mixing_space);

        let hue_0 = match color_0 {
            Color::Hsl(hsl) => hsl.hue,
            Color::Hsv(hsv) => hsv.hue,
            _ => unreachable!(),
        }
        .into_positive_degrees();

        let hue_1 = match color_1 {
            Color::Hsl(hsl) => hsl.hue,
            Color::Hsv(hsv) => hsv.hue,
            _ => unreachable!(),
        }
        .into_positive_degrees();

        // Check if we need to interpolate over the 360° boundary.
        if (hue_0 - hue_1).abs() > 180.0 {
            let hue_0 = if hue_0 < hue_1 { hue_0 + 360.0 } else { hue_0 };
            let hue_1 = if hue_1 < hue_0 { hue_1 + 360.0 } else { hue_1 };

            let hue = hue_0 * (1.0 - t) + hue_1 * t;

            if mixing_space == ColorSpace::Hsl {
                let (_, saturation, lightness, alpha) = out.to_hsl().into_components();
                return Color::Hsl(Hsl::new(hue, saturation, lightness, alpha));
            } else if mixing_space == ColorSpace::Hsv {
                let (_, saturation, value, alpha) = out.to_hsv().into_components();
                return Color::Hsv(Hsv::new(hue, saturation, value, alpha));
            }
        }
    }

    out
}

struct RatioRepr(f32);

impl std::fmt::Display for RatioRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}%", self.0 * 100.0)
    }
}
