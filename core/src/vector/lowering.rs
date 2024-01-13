//! Lowering Typst Document into SvgItem.

use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Read;
use std::sync::Arc;

use comemo::{Track, Tracked};
use once_cell::sync::OnceCell;
use typst::foundations::Smart;
use typst::introspection::{Introspector, Meta};
use typst::layout::{
    Abs as TypstAbs, Dir, Frame, FrameItem, FrameKind, GroupItem, Position, Ratio as TypstRatio,
    Size,
};
use typst::model::{Destination, Document};
use typst::text::{Font, TextItem};
use typst::visualize::{
    FixedStroke, Geometry, Gradient, LineCap, LineJoin, Paint, PathItem, Pattern, Shape,
};
use typst::visualize::{Image, RelativeTo};

use ttf_parser::OutlineBuilder;
use typst::syntax::Span;

use super::ir::PatternItem;
use super::{
    geom::Scalar,
    ir::{self, GlyphItem, ImageGlyphItem, OutlineGlyphItem, SvgItem, TransformItem},
};
use super::{
    path2d::SvgPath2DBuilder,
    sk,
    utils::{AbsExt, ToCssExt},
};
use crate::font::GlyphProvider;
use crate::hash::{Fingerprint, FingerprintHasher, FingerprintSipHasher};
use crate::vector::ir::{ColorItem, GradientItem, GradientKind};
use crate::ImmutStr;
use ttf_parser::GlyphId;

static WARN_VIEW_BOX: OnceCell<()> = OnceCell::new();

type ExtraSvgItems = rpds::RedBlackTreeMapSync<Fingerprint, Arc<SvgItem>>;

/// Lower a frame item into svg item.
pub struct LowerBuilder<'a> {
    introspector: &'a Introspector,
    /// Extra items that used by the document but not directly rendered.
    /// For example, gradients.
    pub extra_items: HashMap<Fingerprint, Arc<SvgItem>>,
}

static LINE_HINT_ELEMENTS: once_cell::sync::Lazy<std::collections::HashSet<&'static str>> =
    once_cell::sync::Lazy::new(|| {
        let mut set = std::collections::HashSet::new();
        set.insert("heading");
        set
    });

impl<'a> LowerBuilder<'a> {
    pub fn new(output: &'a Document) -> Self {
        Self {
            //todo: introspector
            introspector: &output.introspector,
            extra_items: HashMap::new(),
        }
    }

    /// Lower a frame into svg item.
    pub fn lower(&mut self, frame: &Frame) -> SvgItem {
        let introspector = self.introspector.track();
        let (item, extras) = Self::lower_frame(introspector, frame);
        self.extra_items
            .extend(extras.into_iter().map(|t| (*t.0, t.1.clone())));
        item
    }

    /// Lower a frame into svg item.
    // #[comemo::memoize]
    // suboptimal since the frame is not prehashed
    fn lower_frame(introspector: Tracked<Introspector>, frame: &Frame) -> (SvgItem, ExtraSvgItems) {
        let mut items = Vec::with_capacity(frame.items().len());
        let mut extras = ExtraSvgItems::default();

        for (pos, item) in frame.items() {
            let item = match item {
                FrameItem::Group(group) => {
                    let (item, sub_extras) = Self::lower_group(introspector, group);
                    extras = merge_extras(extras, sub_extras);

                    item
                }
                FrameItem::Text(text) => {
                    let (item, sub_extras) = Self::lower_text(introspector, text);
                    extras = merge_extras(extras, sub_extras);

                    item
                }
                FrameItem::Shape(shape, span_id) => {
                    let (s, sub_extras) = Self::lower_shape(introspector, shape);
                    extras = merge_extras(extras, sub_extras);

                    SvgItem::Path((s, span_id_to_u64(span_id)))
                }
                FrameItem::Image(image, size, span_id) => {
                    SvgItem::Image((lower_image(image, *size), span_id_to_u64(span_id)))
                }
                FrameItem::Meta(meta, size) => match meta {
                    Meta::Link(lnk) => match lnk {
                        Destination::Url(url) => Self::lower_link(url, *size),
                        Destination::Position(dest) => Self::lower_position(*dest, *size),
                        Destination::Location(loc) => {
                            // todo: process location before lowering
                            let dest = introspector.position(*loc);
                            Self::lower_position(dest, *size)
                        }
                    },
                    Meta::Elem(elem) => {
                        if !LINE_HINT_ELEMENTS.contains(elem.func().name()) {
                            continue;
                        }

                        SvgItem::ContentHint('\n')
                    }
                    #[cfg(not(feature = "no-content-hint"))]
                    Meta::ContentHint(c) => SvgItem::ContentHint(*c),
                    // todo: support page label
                    Meta::PdfPageLabel(..) | Meta::PageNumbering(..) | Meta::Hide => continue,
                },
            };

            items.push(((*pos).into(), item));
        }

        // swap link items
        items.sort_by(|x, y| {
            let x_is_link = matches!(x.1, SvgItem::Link(..));
            let y_is_link = matches!(y.1, SvgItem::Link(..));
            if x_is_link || y_is_link {
                if x_is_link && y_is_link {
                    return std::cmp::Ordering::Equal;
                } else if x_is_link {
                    return std::cmp::Ordering::Greater;
                } else {
                    return std::cmp::Ordering::Less;
                }
            }

            std::cmp::Ordering::Equal
        });

        let item = match frame.kind() {
            FrameKind::Hard => SvgItem::Group(ir::GroupItem(items), Some(frame.size().into())),
            FrameKind::Soft => SvgItem::Group(ir::GroupItem(items), None),
        };

        (item, extras)
    }

    /// Lower a group frame with optional transform and clipping into svg item.
    // #[comemo::memoize]
    // suboptimal since the frame is not prehashed
    fn lower_group(
        introspector: Tracked<Introspector>,
        group: &GroupItem,
    ) -> (SvgItem, ExtraSvgItems) {
        let (mut inner, extras) = Self::lower_frame(introspector, &group.frame);

        if let Some(p) = group.clip_path.as_ref() {
            // todo: merge
            let mut builder = SvgPath2DBuilder(String::new());

            // to ensure that our shape focus on the original point
            builder.move_to(0., 0.);
            for elem in &p.0 {
                match elem {
                    PathItem::MoveTo(p) => {
                        builder.move_to(p.x.to_f32(), p.y.to_f32());
                    }
                    PathItem::LineTo(p) => {
                        builder.line_to(p.x.to_f32(), p.y.to_f32());
                    }
                    PathItem::CubicTo(p1, p2, p3) => {
                        builder.curve_to(
                            p1.x.to_f32(),
                            p1.y.to_f32(),
                            p2.x.to_f32(),
                            p2.y.to_f32(),
                            p3.x.to_f32(),
                            p3.y.to_f32(),
                        );
                    }
                    PathItem::ClosePath => {
                        builder.close();
                    }
                };
            }
            let d = builder.0.into();

            inner = SvgItem::Transformed(ir::TransformedItem(
                TransformItem::Clip(Arc::new(ir::PathItem {
                    d,
                    size: None,
                    styles: vec![],
                })),
                Box::new(inner),
            ));
        };

        (
            SvgItem::Transformed(ir::TransformedItem(
                TransformItem::Matrix(Arc::new(group.transform.into())),
                Box::new(inner),
            )),
            extras,
        )
    }

    /// Lower a link into svg item.
    pub(super) fn lower_link(url: &str, size: Size) -> ir::SvgItem {
        SvgItem::Link(ir::LinkItem {
            href: url.into(),
            size: size.into(),
        })
    }

    /// Lower a document position into svg item.
    #[comemo::memoize]
    pub(super) fn lower_position(pos: Position, size: Size) -> ir::SvgItem {
        let lnk = ir::LinkItem {
            href: format!(
                "@typst:handleTypstLocation(this, {}, {}, {})",
                pos.page,
                pos.point.x.to_f32(),
                pos.point.y.to_f32()
            )
            .into(),
            size: size.into(),
        };

        SvgItem::Link(lnk)
    }

    /// Lower a text into svg item.
    #[comemo::memoize]
    pub(super) fn lower_text(
        introspector: Tracked<Introspector>,
        text: &TextItem,
    ) -> (SvgItem, ExtraSvgItems) {
        let mut glyphs = Vec::with_capacity(text.glyphs.len());
        for glyph in &text.glyphs {
            let id = GlyphId(glyph.id);
            glyphs.push((
                glyph.x_offset.at(text.size).into(),
                glyph.x_advance.at(text.size).into(),
                ir::GlyphItem::Raw(text.font.clone(), id),
            ));
        }

        let glyph_chars: String = text.text.to_string();
        let mut extras = ExtraSvgItems::default();

        // let fill = text.fill.clone().to_css().into();
        let fill = Self::lower_paint(introspector, &text.fill, &mut extras);

        let span_id = text
            .glyphs
            .iter()
            .filter(|g| g.span.0 != Span::detached())
            .map(|g| &g.span.0)
            .map(span_id_to_u64)
            .max()
            .unwrap_or_else(|| span_id_to_u64(&Span::detached()));

        (
            SvgItem::Text(ir::TextItem {
                font: text.font.clone(),
                content: Arc::new(ir::TextItemContent {
                    content: glyph_chars.into(),
                    glyphs,
                    span_id,
                }),
                shape: Arc::new(ir::TextShape {
                    size: Scalar(text.size.to_f32()),
                    dir: match text.lang.dir() {
                        Dir::LTR => "ltr",
                        Dir::RTL => "rtl",
                        Dir::TTB => "ttb",
                        Dir::BTT => "btt",
                    }
                    .into(),
                    fill,
                }),
            }),
            extras,
        )
    }

    /// Lower a geometrical shape into svg item.
    #[comemo::memoize]
    fn lower_shape(
        introspector: Tracked<Introspector>,
        shape: &Shape,
    ) -> (ir::PathItem, ExtraSvgItems) {
        let mut builder = SvgPath2DBuilder(String::new());
        let mut extras = ExtraSvgItems::default();

        // to ensure that our shape focus on the original point
        builder.move_to(0., 0.);
        match shape.geometry {
            Geometry::Line(target) => {
                builder.line_to(target.x.to_f32(), target.y.to_f32());
            }
            Geometry::Rect(size) => {
                let w = size.x.to_f32();
                let h = size.y.to_f32();
                builder.line_to(0., h);
                builder.line_to(w, h);
                builder.line_to(w, 0.);
                builder.close();
            }
            Geometry::Path(ref path) => {
                for elem in &path.0 {
                    match elem {
                        PathItem::MoveTo(p) => {
                            builder.move_to(p.x.to_f32(), p.y.to_f32());
                        }
                        PathItem::LineTo(p) => {
                            builder.line_to(p.x.to_f32(), p.y.to_f32());
                        }
                        PathItem::CubicTo(p1, p2, p3) => {
                            builder.curve_to(
                                p1.x.to_f32(),
                                p1.y.to_f32(),
                                p2.x.to_f32(),
                                p2.y.to_f32(),
                                p3.x.to_f32(),
                                p3.y.to_f32(),
                            );
                        }
                        PathItem::ClosePath => {
                            builder.close();
                        }
                    };
                }
            }
        };

        let d = builder.0.into();

        let mut styles = Vec::new();

        if let Some(paint_fill) = &shape.fill {
            styles.push(ir::PathStyle::Fill(Self::lower_paint(
                introspector,
                paint_fill,
                &mut extras,
            )));
        }

        // todo: default miter_limit, thickness
        if let Some(FixedStroke {
            paint,
            thickness,
            line_cap,
            line_join,
            dash_pattern,
            miter_limit,
        }) = &shape.stroke
        {
            if let Some(pattern) = dash_pattern.as_ref() {
                styles.push(ir::PathStyle::StrokeDashOffset(pattern.phase.into()));
                let d = pattern.array.clone();
                let d = d.into_iter().map(Scalar::from).collect();
                styles.push(ir::PathStyle::StrokeDashArray(d));
            }

            styles.push(ir::PathStyle::StrokeWidth((*thickness).into()));
            styles.push(ir::PathStyle::StrokeMitterLimit((*miter_limit).into()));
            match line_cap {
                LineCap::Butt => {}
                LineCap::Round => styles.push(ir::PathStyle::StrokeLineCap("round".into())),
                LineCap::Square => styles.push(ir::PathStyle::StrokeLineCap("square".into())),
            };
            match line_join {
                LineJoin::Miter => {}
                LineJoin::Bevel => styles.push(ir::PathStyle::StrokeLineJoin("bevel".into())),
                LineJoin::Round => styles.push(ir::PathStyle::StrokeLineJoin("round".into())),
            }

            styles.push(ir::PathStyle::Stroke(Self::lower_paint(
                introspector,
                paint,
                &mut extras,
            )));
        }

        let mut shape_size = shape.geometry.bbox_size();
        // Edge cases for strokes.
        if shape_size.x.to_pt() == 0.0 {
            shape_size.x = TypstAbs::pt(1.0);
        }

        if shape_size.y.to_pt() == 0.0 {
            shape_size.y = TypstAbs::pt(1.0);
        }

        let item = ir::PathItem {
            d,
            size: Some(shape_size.into()),
            styles,
        };

        (item, extras)
    }

    #[inline]
    pub(super) fn lower_paint(
        introspector: Tracked<Introspector>,
        g: &Paint,
        cell: &mut ExtraSvgItems,
    ) -> ImmutStr {
        match g {
            Paint::Solid(c) => c.to_css().into(),
            Paint::Pattern(e) => {
                let (g, fingerprint) = Self::lower_pattern(introspector, e, cell);
                *cell = cell.insert(fingerprint, Arc::new(ir::SvgItem::Pattern(g)));
                format!("@{}", fingerprint.as_svg_id("p")).into()
            }
            Paint::Gradient(g) => {
                let (g, fingerprint) = Self::lower_graident(g);
                *cell = cell.insert(fingerprint, Arc::new(ir::SvgItem::Gradient(Arc::new(g))));
                format!("@{}", fingerprint.as_svg_id("g")).into()
            }
        }
    }

    #[comemo::memoize]
    pub(super) fn lower_graident(g: &Gradient) -> (GradientItem, Fingerprint) {
        let mut stops = Vec::with_capacity(g.stops_ref().len());
        for (c, step) in g.stops_ref() {
            let [r, g, b, a] = c.to_vec4_u8();
            stops.push((ColorItem { r, g, b, a }, (*step).into()))
        }

        let relative_to_self = match g.relative() {
            Smart::Auto => None,
            Smart::Custom(t) => Some(t == RelativeTo::Self_),
        };

        let anti_alias = g.anti_alias();
        let space = g.space().into();

        let mut styles = Vec::new();
        let kind = match g {
            Gradient::Linear(l) => GradientKind::Linear(l.angle.into()),
            Gradient::Radial(l) => {
                if l.center.x != TypstRatio::new(0.5) || l.center.y != TypstRatio::new(0.5) {
                    styles.push(ir::GradientStyle::Center(l.center.into()));
                }

                if l.focal_center.x != TypstRatio::new(0.5)
                    || l.focal_center.y != TypstRatio::new(0.5)
                {
                    styles.push(ir::GradientStyle::FocalCenter(l.focal_center.into()));
                }

                if l.focal_radius != TypstRatio::zero() {
                    styles.push(ir::GradientStyle::FocalRadius(l.focal_radius.into()));
                }

                GradientKind::Radial(l.radius.into())
            }
            Gradient::Conic(l) => {
                if l.center.x != TypstRatio::new(0.5) || l.center.y != TypstRatio::new(0.5) {
                    styles.push(ir::GradientStyle::Center(l.center.into()));
                }

                GradientKind::Conic(l.angle.into())
            }
        };

        let g = GradientItem {
            stops,
            relative_to_self,
            anti_alias,
            space,
            kind,
            styles,
        };

        // todo: don't leak the fingerprint primitive
        let mut f = FingerprintSipHasher::default();
        g.type_id().hash(&mut f);
        g.hash(&mut f);
        let (f, _) = f.finish_fingerprint();

        (g, f)
    }

    pub(super) fn lower_pattern(
        introspector: Tracked<Introspector>,
        g: &Pattern,
        cell: &mut ExtraSvgItems,
    ) -> (PatternItem, Fingerprint) {
        let (frame, extras) = Self::lower_frame(introspector, g.frame());
        *cell = merge_extras(extras, cell.clone());

        let relative_to_self = match g.relative() {
            Smart::Auto => None,
            Smart::Custom(t) => Some(t == RelativeTo::Self_),
        };

        let pattern = PatternItem {
            frame: Arc::new(frame),
            size: g.size().into(),
            spacing: g.spacing().into(),
            relative_to_self,
        };

        // todo: don't leak the fingerprint primitive
        let mut f = FingerprintSipHasher::default();
        pattern.type_id().hash(&mut f);
        g.hash(&mut f);
        let (f, _) = f.finish_fingerprint();

        (pattern, f)
    }
}

/// Lower a glyph into svg item.
pub struct GlyphLowerBuilder<'a> {
    gp: &'a GlyphProvider,

    /// Whether to lower ligature information
    lowering_ligature: bool,
}

impl<'a> GlyphLowerBuilder<'a> {
    pub fn new(gp: &'a GlyphProvider, lowering_ligature: bool) -> Self {
        Self {
            gp,
            lowering_ligature: cfg!(feature = "experimental-ligature") && lowering_ligature,
        }
    }

    pub fn lower_glyph(&self, glyph_item: &GlyphItem) -> Option<GlyphItem> {
        match glyph_item {
            GlyphItem::Raw(font, id) => {
                let id = *id;
                self.lower_svg_glyph(font, id)
                    .map(GlyphItem::Image)
                    .or_else(|| self.lower_bitmap_glyph(font, id).map(GlyphItem::Image))
                    .or_else(|| self.lower_outline_glyph(font, id).map(GlyphItem::Outline))
            }
            GlyphItem::Image(..) | GlyphItem::Outline(..) => Some(glyph_item.clone()),
            GlyphItem::None => Some(GlyphItem::None),
        }
    }

    fn ligature_len(&self, font: &Font, id: GlyphId) -> u8 {
        if !self.lowering_ligature {
            return 0;
        }

        self.gp
            .ligature_glyph(font, id)
            .map(|l| l.len())
            .unwrap_or_default() as u8
    }

    /// Lower an SVG glyph into svg item.
    /// More information: https://learn.microsoft.com/zh-cn/typography/opentype/spec/svg
    fn lower_svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ImageGlyphItem>> {
        let image = extract_svg_glyph(self.gp, font, id)?;

        // position our image
        let ascender = font
            .metrics()
            .ascender
            .at(typst::layout::Abs::raw(font.metrics().units_per_em))
            .to_f32();

        Some(Arc::new(ImageGlyphItem {
            ts: ir::Transform {
                sx: Scalar(1.),
                ky: Scalar(0.),
                kx: Scalar(0.),
                sy: Scalar(-1.),
                tx: Scalar(0.),
                ty: Scalar(ascender),
            },
            image,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower a bitmap glyph into the svg text.
    fn lower_bitmap_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<ImageGlyphItem>> {
        let ppem = u16::MAX;
        let upem = font.metrics().units_per_em as f32;

        let (glyph_image, raster_x, raster_y) = self.gp.bitmap_glyph(font, id, ppem)?;

        // FIXME: Vertical alignment isn't quite right for Apple Color Emoji,
        // and maybe also for Noto Color Emoji. And: Is the size calculation
        // correct?

        let w = glyph_image.width() as f64;
        let h = glyph_image.height() as f64;
        let sz = Size::new(typst::layout::Abs::raw(w), typst::layout::Abs::raw(h));

        let image = ir::ImageItem {
            image: Arc::new(glyph_image.into()),
            size: sz.into(),
        };

        // position our image
        // first, the ascender is used
        // next, also apply an offset of (1 - ascender) like typst
        let adjusted = font.metrics().ascender * 2. - typst::layout::Em::one();
        // let adjusted = font.metrics().ascender;

        let adjusted = adjusted
            .at(typst::layout::Abs::raw(font.metrics().units_per_em))
            .to_f32();

        let ts = sk::Transform::from_scale(upem / w as f32, -upem / h as f32);

        // size
        let dx = raster_x as f32;
        let dy = raster_y as f32;
        let ts = ts.post_translate(dx, adjusted + dy);

        Some(Arc::new(ImageGlyphItem {
            ts: ts.into(),
            image,
            ligature_len: self.ligature_len(font, id),
        }))
    }

    /// Lower an outline glyph into svg text. This is the "normal" case.
    fn lower_outline_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<OutlineGlyphItem>> {
        let d = self.gp.outline_glyph(font, id)?.into();

        Some(Arc::new(OutlineGlyphItem {
            ts: None,
            d,
            ligature_len: self.ligature_len(font, id),
        }))
    }
}

fn extract_svg_glyph(g: &GlyphProvider, font: &Font, id: GlyphId) -> Option<ir::ImageItem> {
    let data = g.svg_glyph(font, id)?;
    let mut data = data.as_ref();

    let font_metrics = font.metrics();

    // Decompress SVGZ.
    let mut decoded = vec![];

    // The first three bytes of the gzip-encoded document header must be
    //   0x1F, 0x8B, 0x08.
    if data.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = flate2::read::GzDecoder::new(data);
        decoder.read_to_end(&mut decoded).ok()?;
        data = &decoded;
    }

    // todo: It is also legal to provide a SVG document containing multiple glyphs.
    // > When a font engine renders glyph 14, the result shall be the same as
    // > rendering the following SVG document:
    // > `  <svg> <defs> <use #glyph{id}> </svg>`
    // See: <https://learn.microsoft.com/en-us/typography/opentype/spec/svg#glyph-identifiers>

    let upem = typst::layout::Abs::raw(font.units_per_em());
    let (width, height) = (upem.to_f32(), upem.to_f32());
    let origin_ascender = font_metrics.ascender.at(upem).to_f32();

    let doc_string = String::from_utf8(data.to_owned()).unwrap();

    // todo: verify SVG capability requirements and restrictions

    // Partially parse the view box attribute
    let mut svg_str = std::str::from_utf8(data).ok()?.to_owned();
    let FindViewBoxResult {
        start_span,
        first_viewbox,
    } = find_viewbox_attr(svg_str.as_str());

    // determine view box
    let view_box = first_viewbox.as_ref()
        .map(|s| {
            WARN_VIEW_BOX.get_or_init(|| {
                println!(
                    "render_svg_glyph with viewBox, This should be helpful if you can help us verify the result: {:?} {:?}",
                    font.info().family,
                    doc_string
                );
            });
            s.1.as_str().to_owned()
        })
        .unwrap_or_else(|| format!("0 {} {width} {height}", -origin_ascender));

    // determine view box
    match first_viewbox {
        Some((span, ..)) => {
            // replace the first viewBox attribute
            svg_str.replace_range(span.range(), format!(r#"viewBox="{view_box}""#).as_str());
        }
        None => {
            // insert viewBox attribute to the begin of svg tag
            svg_str.insert_str(
                start_span.unwrap().range().end,
                format!(r#" viewBox="{view_box}""#).as_str(),
            );
        }
    }

    let glyph_image = typst::visualize::Image::new(
        svg_str.as_bytes().to_vec().into(),
        typst::visualize::ImageFormat::Vector(typst::visualize::VectorFormat::Svg),
        // typst::geom::Axes::new(width as u32, height as u32),
        None,
    )
    .ok()?;

    let sz = Size::new(
        typst::layout::Abs::raw(glyph_image.width() as f64),
        typst::layout::Abs::raw(glyph_image.height() as f64),
    );

    Some(ir::ImageItem {
        image: Arc::new(glyph_image.into()),
        size: sz.into(),
    })
}

/// Lower a raster or SVG image into svg item.
#[comemo::memoize]
fn lower_image(image: &Image, size: Size) -> ir::ImageItem {
    ir::ImageItem {
        image: Arc::new(image.clone().into()),
        size: size.into(),
    }
}

struct FindViewBoxResult<'a> {
    start_span: Option<xmlparser::StrSpan<'a>>,
    first_viewbox: Option<(xmlparser::StrSpan<'a>, xmlparser::StrSpan<'a>)>,
}

/// Find the string location of the **first** viewBox attribute.
/// When there are multiple viewBox attributes, the first one is used (as many
/// xml-based dom engines do).
fn find_viewbox_attr(svg_str: &'_ str) -> FindViewBoxResult<'_> {
    let document = xmlparser::Tokenizer::from(svg_str);

    let mut start_span = None;
    let mut first_viewbox = None;
    for n in document {
        let tok = n.unwrap();
        match tok {
            xmlparser::Token::ElementStart { span, local, .. } => {
                if local.as_str() == "svg" {
                    start_span = Some(span);
                }
            }
            xmlparser::Token::Attribute {
                span, local, value, ..
            } => {
                if local.as_str() == "viewBox" {
                    first_viewbox = Some((span, value));
                    break;
                }
            }
            xmlparser::Token::ElementEnd { .. } => break,
            _ => {}
        }
    }

    FindViewBoxResult {
        start_span,
        first_viewbox,
    }
}

fn merge_extras(mut extras: ExtraSvgItems, mut other: ExtraSvgItems) -> ExtraSvgItems {
    // swap ensure O(n log n) complexity for merging
    if extras.size() < other.size() {
        std::mem::swap(&mut extras, &mut other);
    }

    for (f, item) in other.into_iter() {
        extras = extras.insert(*f, item.clone());
    }
    extras
}

const DETACHED: u64 = 1;
const SPAN_BITS: u64 = 48;

// todo: more safe way to transfer span id across process
/// Note: this function may be removed in the future.
pub fn span_id_to_u64(span_id: &Span) -> u64 {
    span_id
        .id()
        .map(|file_id| ((file_id.into_raw() as u64) << SPAN_BITS) | span_id.number())
        .unwrap_or(DETACHED)
}

/// Note: this function may be removed in the future.
pub fn span_id_from_u64(span_id: u64) -> Option<Span> {
    use typst::syntax::FileId;
    let file_id = if span_id == DETACHED {
        return Some(Span::detached());
    } else {
        let file_id = (span_id >> SPAN_BITS) as u16;
        FileId::from_raw(file_id)
    };

    Span::new(file_id, span_id & ((1u64 << SPAN_BITS) - 1))
}

#[cfg(test)]
mod tests {
    use typst::syntax::FileId;
    use typst::syntax::Span;

    use super::DETACHED;
    use super::SPAN_BITS;
    use super::{span_id_from_u64, span_id_to_u64};

    #[test]
    fn test_convert_span_id_u64() {
        let file_id = FileId::from_raw(1);
        let span_id = Span::new(file_id, 2).unwrap();

        // test span -> u64
        assert_eq!(span_id_to_u64(&Span::detached()), DETACHED);
        assert_eq!(span_id_to_u64(&span_id), (1u64 << SPAN_BITS) | 2);

        // test u64 -> span
        assert_eq!(span_id_from_u64(DETACHED), Some(Span::detached()));
        assert_eq!(span_id_from_u64(span_id_to_u64(&span_id)), Some(span_id));

        // test invalid u64
        assert_eq!(span_id_from_u64(0), None);
    }
}
