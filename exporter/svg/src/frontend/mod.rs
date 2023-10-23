use std::{collections::HashSet, f32::consts::TAU, fmt::Write, sync::Arc};

use typst::{
    doc::Document,
    geom::{Angle, Color, ColorSpace, Hsl, Hsv, Quadrant, WeightedColor},
};
use typst_ts_core::{
    font::GlyphProvider,
    hash::{item_hash128, Fingerprint, FingerprintBuilder},
    vector::{
        flat_ir::{self, ModuleBuilder},
        ir::{
            Axes, DefId, GlyphItem, GlyphPackBuilder, GradientItem, GradientKind, GradientStyle,
            Scalar, Size, SvgItem,
        },
        vm::{RenderState, RenderVm},
        LowerBuilder,
    },
};

pub(crate) mod context;
use context::{ClipPathMap, RenderContext, StyleDefMap};

#[cfg(feature = "flat-vector")]
pub(crate) mod dynamic_layout;
pub use dynamic_layout::DynamicLayoutSvgExporter;
#[cfg(feature = "flat-vector")]
pub(crate) mod flat;
#[cfg(feature = "flat-vector")]
pub(crate) mod incremental;
use crate::{
    backend::{SvgGlyphBuilder, SvgText, SvgTextNode},
    utils::AbsExt,
    ExportFeature,
};
pub use incremental::{IncrSvgDocClient, IncrSvgDocServer, IncrementalRenderContext};

use self::context::GradientMap;

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
    /// Render the header of SVG.
    /// <svg> .. </svg>
    /// ^^^^^
    fn header_inner(w: f32, h: f32) -> String {
        format!(
            r#"<svg class="typst-doc" viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            w, h, w, h,
        )
    }

    /// Render the header of SVG for [`Document`].
    /// <svg> .. </svg>
    /// ^^^^^
    fn header_doc(output: &Document) -> String {
        // calculate the width and height of SVG
        let w = output
            .pages
            .iter()
            .map(|p| p.size().x.to_f32().ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let h = output
            .pages
            .iter()
            .map(|p| p.size().y.to_f32().ceil())
            .sum::<f32>();

        Self::header_inner(w, h)
    }

    /// Render the style for SVG
    /// <svg> <style/> .. </svg>
    ///       ^^^^^^^^
    /// See [`StyleDefMap`].
    fn style_defs(style_defs: StyleDefMap, svg: &mut Vec<SvgText>) {
        // style defs
        svg.push(r#"<style type="text/css">"#.into());

        // sort and push the style defs
        let mut style_defs = style_defs.into_iter().collect::<Vec<_>>();
        style_defs.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(style_defs.into_iter().map(|v| SvgText::Plain(v.1)));

        svg.push("</style>".into());
    }

    /// Render the clip paths for SVG
    /// <svg> <defs> <clipPath/> </defs> .. </svg>
    ///              ^^^^^^^^^^^
    /// See [`ClipPathMap`].
    fn clip_paths(clip_paths: ClipPathMap, svg: &mut Vec<SvgText>) {
        let mut clip_paths = clip_paths.into_iter().collect::<Vec<_>>();
        clip_paths.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in clip_paths {
            svg.push(SvgText::Plain(format!(
                r##"<clipPath id="{}"><path d="{}"/></clipPath>"##,
                id.as_svg_id("c"),
                clip_path
            )));
        }
    }

    /// Render the gradients for SVG
    /// <svg> <defs> <gradient/> </defs> .. </svg>
    ///              ^^^^^^^^^^^
    /// See [`GradientMap`].
    fn gradients<'a>(
        gradients: impl Iterator<Item = (&'a Fingerprint, &'a GradientItem)>,
        svg: &mut Vec<SvgText>,
    ) {
        let mut sub_gradients = HashSet::<(Fingerprint, SVGSubGradient)>::default();

        // todo: aspect ratio
        for (id, gradient) in gradients {
            match &gradient.kind {
                GradientKind::Linear(angle) => {
                    // todo: use native angle
                    let angle = typst::geom::Angle::rad(angle.0 as f64);

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
                    start_c.clone().to_hex(),
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
                        c.to_hex(),
                    )));
                }

                svg.push(SvgText::Plain(format!(
                    r##"<stop offset="{}" stop-color="{}"/>"##,
                    RatioRepr(end_t.0),
                    end_c.clone().to_hex(),
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
                gradient.c0.to_hex(),
            )));

            svg.push(SvgText::Plain(format!(
                r##"<stop offset="1" stop-color="{}"/>"##,
                gradient.c1.to_hex(),
            )));

            svg.push(SvgText::Plain("</linearGradient>".to_owned()));
        }
    }

    /// Template SVG.
    fn render_svg_template<'a>(
        t: SvgTask<Feat>,
        header: String,
        mut body: Vec<SvgText>,
        glyphs: impl IntoIterator<Item = SvgText>,
        gradients: impl Iterator<Item = (&'a Fingerprint, &'a GradientItem)>,
    ) -> Vec<SvgText> {
        let mut svg = vec![
            SvgText::Plain(header),
            // base style
        ];

        if Feat::WITH_BUILTIN_CSS {
            svg.push(r#"<style type="text/css">"#.into());
            svg.push(include_str!("./typst.svg.css").into());
            svg.push("</style>".into());
        }

        // attach the glyph defs, clip paths, and style defs
        svg.push(r#"<defs class="glyph">"#.into());
        svg.extend(glyphs);
        svg.push("</defs>".into());
        svg.push(r#"<defs class="clip-path">"#.into());
        Self::clip_paths(t.clip_paths, &mut svg);
        Self::gradients(gradients, &mut svg);
        svg.push("</defs>".into());
        Self::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut body);

        if Feat::WITH_RESPONSIVE_JS {
            // attach the javascript for animations
            svg.push(r#"<script type="text/javascript">"#.into());
            svg.push(include_str!("./typst.svg.js").into());
            svg.push("</script>".into());
        }

        // close SVG
        svg.push("</svg>".into());

        svg
    }

    /// Render SVG for [`Document`].
    /// It does not flatten the vector items before rendering so called
    /// "transient".
    pub(crate) fn render_transient_svg(output: &Document) -> Vec<SvgText> {
        let mut t = SvgTask::<Feat>::default();

        // render SVG header
        let header = Self::header_doc(output);

        // lowering the document into svg items
        let mut lower_builder = LowerBuilder::new(output);
        let pages = output
            .pages
            .iter()
            .map(|p| lower_builder.lower(p))
            .collect::<Vec<_>>();
        let mut module = ModuleBuilder::default();

        for (_, ext) in lower_builder.extra_items.clone().into_iter() {
            module.build(ext);
        }

        let module = module.finalize();

        // render SVG body
        let mut svg_body = vec![];
        t.render_pages_transient(module, output, pages, &mut svg_body);

        // render the glyphs collected from the pages
        let (_, glyphs) = std::mem::take(&mut t.glyph_defs).finalize();
        let glyphs = t.render_glyphs(glyphs.iter().enumerate().map(|(x, (_, y))| (x, y)), false);

        let gradients = lower_builder
            .extra_items
            .iter()
            .filter_map(|(f, item)| match item {
                SvgItem::Gradient(item) => Some((f, item)),
                _ => None,
            });

        // template SVG
        Self::render_svg_template(t, header, svg_body, glyphs, gradients)
    }

    /// Render SVG wrapped with HTML for [`Document`].
    /// It does not flatten the vector items before rendering so called
    /// "transient".
    pub(crate) fn render_transient_html(output: &Document) -> Vec<SvgText> {
        // render SVG
        let mut svg = Self::render_transient_svg(output);

        // wrap SVG with html
        let mut html: Vec<SvgText> = Vec::with_capacity(svg.len() + 3);
        html.push(r#"<html><head><meta charset="utf-8" /><title>"#.into());
        html.push(SvgText::Plain(
            output
                .title
                .clone()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Typst Document".into()),
        ));
        html.push(r#"</title></head><body>"#.into());
        html.append(&mut svg);
        html.push(r#"</body></html>"#.into());

        html
    }
}

/// The task context for exporting svg.
/// It is also as a namespace for all the functions used in the task.
pub struct SvgTask<Feat: ExportFeature> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    pub(crate) glyph_provider: GlyphProvider,

    /// A fingerprint builder for generating unique id.
    pub(crate) fingerprint_builder: FingerprintBuilder,

    /// Stores the glyphs used in the document.
    pub(crate) glyph_defs: GlyphPackBuilder,
    /// Stores the style definitions used in the document.
    pub(crate) style_defs: StyleDefMap,
    /// Stores the clip paths used in the document.
    pub(crate) clip_paths: ClipPathMap,
    /// Stores the gradients used in the document.
    pub(crate) gradients: GradientMap,

    _feat_phantom: std::marker::PhantomData<Feat>,
}

/// Unfortunately, `Default` derive does not work for generic structs.
impl<Feat: ExportFeature> Default for SvgTask<Feat> {
    fn default() -> Self {
        Self {
            glyph_provider: GlyphProvider::default(),

            fingerprint_builder: FingerprintBuilder::default(),

            glyph_defs: GlyphPackBuilder::default(),
            style_defs: StyleDefMap::default(),
            clip_paths: ClipPathMap::default(),
            gradients: GradientMap::default(),

            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// Sets the glyph provider for task.
    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

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
    #[cfg(feature = "flat-vector")]
    pub fn get_render_context<'m, 't>(
        &'t mut self,
        module: &'m flat_ir::Module,
    ) -> RenderContext<'m, 't, Feat> {
        RenderContext::<Feat> {
            glyph_provider: self.glyph_provider.clone(),

            module,

            fingerprint_builder: &mut self.fingerprint_builder,

            glyph_defs: &mut self.glyph_defs,
            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,
            gradients: &mut self.gradients,

            should_attach_debug_info: Feat::SHOULD_ATTACH_DEBUG_INFO,
            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }

    /// fork a render task.
    #[cfg(not(feature = "flat-vector"))]
    pub fn get_render_context<'m>(&mut self) -> RenderContext<'m, '_, Feat> {
        RenderContext::<Feat> {
            glyph_provider: self.glyph_provider.clone(),

            fingerprint_builder: &mut self.fingerprint_builder,

            glyph_defs: &mut self.glyph_defs,
            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,

            should_attach_debug_info: Feat::SHOULD_ATTACH_DEBUG_INFO,
            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
            _m_phantom: Default::default(),
        }
    }

    /// Render glyphs into the svg_body.
    pub(crate) fn render_glyphs<'a, I: Iterator<Item = (usize, &'a GlyphItem)>>(
        &mut self,
        glyphs: I,
        use_stable_glyph_id: bool,
    ) -> Vec<SvgText> {
        let mut render_task = SvgGlyphBuilder {
            glyph_provider: self.glyph_provider.clone(),
        };

        let mut svg_body = Vec::new();

        for (abs_ref, item) in glyphs {
            let glyph_id = if Feat::USE_STABLE_GLYPH_ID && use_stable_glyph_id {
                item.get_fingerprint().as_svg_id("g")
            } else {
                (DefId(abs_ref as u64)).as_svg_id("g")
            };
            svg_body.push(SvgText::Plain(
                render_task
                    .render_glyph(&glyph_id, item)
                    .unwrap_or_default(),
            ))
        }

        svg_body
    }

    /// Render pages into the svg_body.
    pub fn render_pages_transient(
        &mut self,
        module: flat_ir::Module,
        output: &Document,
        pages: Vec<SvgItem>,
        svg_body: &mut Vec<SvgText>,
    ) {
        let mut render_task = self.get_render_context(&module);

        render_task.use_stable_glyph_id = false;

        // accumulate the height of pages
        let mut acc_height = 0u32;
        for (idx, page) in pages.iter().enumerate() {
            let size_f32 = output.pages[idx].size().into();
            let size = Self::page_size(size_f32);

            let attributes = vec![
                ("transform", format!("translate(0, {})", acc_height)),
                ("data-page-width", size.x.to_string()),
                ("data-page-height", size.y.to_string()),
            ];

            let page_svg = render_task.render_item(RenderState::new_size(size_f32), page);

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes,
                content: vec![SvgText::Content(page_svg)],
            })));
            acc_height += size.y;
        }
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
        write!(&mut self.0, "M {} {} ", x, y).unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "L {} {} ", x, y).unwrap();
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

    let mixing_space = gradient.space.into();
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
    let col_0 = col_0.typst();
    let col_1 = col_1.typst();

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
        let hue_0 = col_0.to_space(mixing_space).to_vec4()[0];
        let hue_1 = col_1.to_space(mixing_space).to_vec4()[0];

        // Check if we need to interpolate over the 360° boundary.
        if (hue_0 - hue_1).abs() > 180.0 {
            let hue_0 = if hue_0 < hue_1 { hue_0 + 360.0 } else { hue_0 };
            let hue_1 = if hue_1 < hue_0 { hue_1 + 360.0 } else { hue_1 };

            let hue = hue_0 * (1.0 - t) + hue_1 * t;

            if mixing_space == ColorSpace::Hsl {
                let [_, saturation, lightness, alpha] = out.to_hsl().to_vec4();
                return Color::Hsl(Hsl::new(hue, saturation, lightness, alpha));
            } else if mixing_space == ColorSpace::Hsv {
                let [_, saturation, value, alpha] = out.to_hsv().to_vec4();
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
