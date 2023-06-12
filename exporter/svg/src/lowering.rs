//! Lowering Typst Document into SvgItem.

use std::sync::Arc;

use typst::doc::{Destination, Frame, FrameItem, GroupItem, Meta, Position, TextItem};
use typst::geom::{Geometry, LineCap, LineJoin, Paint, PathItem, Scalar, Shape, Size, Stroke};
use typst::image::Image;

use ttf_parser::OutlineBuilder;

use crate::{
    ir,
    ir::{SvgItem, TransformItem},
    svg::SvgPath2DBuilder,
    utils::{AbsExt, ToCssExt},
    RenderFeature, SvgTask,
};
use ttf_parser::GlyphId;

impl<Feat: RenderFeature> SvgTask<Feat> {
    /// Lower a frame into svg item.
    pub fn lower(&mut self, frame: &Frame) -> SvgItem {
        self.lower_frame(frame)
    }

    /// Lower a frame into svg item.
    fn lower_frame(&mut self, frame: &Frame) -> SvgItem {
        let mut items = Vec::with_capacity(frame.items().len());

        for (pos, item) in frame.items() {
            let item = match item {
                FrameItem::Group(group) => self.lower_group(group),
                FrameItem::Text(text) => Self::lower_text(text),
                FrameItem::Shape(shape, _) => Self::lower_shape(shape),
                FrameItem::Image(image, size, _) => Self::lower_image(image, *size),
                FrameItem::Meta(meta, size) => match meta {
                    Meta::Link(lnk) => match lnk {
                        Destination::Url(url) => self.lower_link(url, *size),
                        Destination::Position(dest) => Self::lower_position(*dest, *size),
                        Destination::Location(loc) => {
                            // todo: process location before lowering
                            let dest = self.annotation_proc.process_location(*loc);
                            Self::lower_position(dest, *size)
                        }
                    },
                    Meta::Elem(..) | Meta::PageNumbering(..) | Meta::Hide => continue,
                },
            };

            items.push((*pos, item));
        }

        SvgItem::Group(ir::GroupItem(items))
    }

    /// Lower a group frame with optional transform and clipping into svg item.
    fn lower_group(&mut self, group: &GroupItem) -> SvgItem {
        let mut inner = self.lower_frame(&group.frame);

        if group.clips {
            let mask_box = {
                let mut builder = SvgPath2DBuilder::default();

                // build a rectangle path
                let size = group.frame.size();
                let w = size.x.to_f32();
                let h = size.y.to_f32();
                builder.rect(0., 0., w, h);

                builder.0
            };

            inner = SvgItem::Transformed(ir::TransformedItem(
                TransformItem::Clip(Arc::new(ir::PathItem {
                    d: mask_box.into(),
                    styles: vec![],
                })),
                Box::new(inner),
            ));
        };

        SvgItem::Transformed(ir::TransformedItem(
            TransformItem::Matrix(Arc::new(group.transform)),
            Box::new(inner),
        ))
    }

    /// Lower a raster or SVG image into svg item.
    #[comemo::memoize]
    pub(super) fn lower_image(image: &Image, size: Size) -> SvgItem {
        SvgItem::Image(ir::ImageItem {
            image: image.clone(),
            size,
        })
    }

    pub(super) fn lower_link(&self, url: &str, size: Size) -> ir::SvgItem {
        let lnk = ir::LinkItem {
            href: url.into(),
            size,
        };

        SvgItem::Link(lnk)
    }

    #[comemo::memoize]
    pub(super) fn lower_position(pos: Position, size: Size) -> ir::SvgItem {
        let lnk = ir::LinkItem {
            href: format!(
                "?locator=page{}&x={}&y={}",
                pos.page,
                pos.point.x.to_f32(),
                pos.point.y.to_f32()
            )
            .into(),
            size,
        };

        SvgItem::Link(lnk)
    }

    /// Lower a text into svg item.
    #[comemo::memoize]
    pub(super) fn lower_text(text: &TextItem) -> SvgItem {
        let mut glyphs = Vec::with_capacity(text.glyphs.len());
        for glyph in &text.glyphs {
            let id = GlyphId(glyph.id);
            glyphs.push((
                glyph.x_offset.at(text.size),
                glyph.x_advance.at(text.size),
                crate::ir::GlyphItem::Raw(text.font.clone(), id),
            ));
        }

        let glyph_chars: String = text.text
            [text.glyphs[0].range().start..text.glyphs[text.glyphs.len() - 1].range().end]
            .to_string();

        let Paint::Solid(fill) = text.fill;
        let fill = fill.to_css().into();

        let ppem = {
            let pixel_per_unit: f32 = text.size.to_f32();
            let units_per_em = text.font.units_per_em() as f32;
            pixel_per_unit / units_per_em
        };

        SvgItem::Text(ir::TextItem {
            content: Arc::new(ir::TextItemContent {
                content: glyph_chars.into(),
                glyphs,
            }),
            shape: Arc::new(ir::TextShape {
                dir: text.lang.dir(),
                ascender: text.font.metrics().ascender.at(text.size),
                upem: Scalar::from(text.font.units_per_em()),
                ppem: Scalar::from(ppem as f64),
                fill,
            }),
        })
    }

    /// Lower a geometrical shape into svg item.
    #[comemo::memoize]
    pub(super) fn lower_shape(shape: &Shape) -> SvgItem {
        let mut builder = SvgPath2DBuilder(String::new());

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
            let Paint::Solid(color) = paint_fill;
            let c = color.to_css();

            styles.push(ir::PathStyle::Fill(c.into()));
        }

        // todo: default miter_limit, thickness
        if let Some(Stroke {
            paint,
            thickness,
            line_cap,
            line_join,
            dash_pattern,
            miter_limit,
        }) = &shape.stroke
        {
            if let Some(pattern) = dash_pattern.as_ref() {
                styles.push(ir::PathStyle::StrokeDashOffset(pattern.phase));
                styles.push(ir::PathStyle::StrokeDashArray(pattern.array.clone().into()));
            }

            styles.push(ir::PathStyle::StrokeWidth(*thickness));
            styles.push(ir::PathStyle::StrokeMitterLimit(*miter_limit));
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

            // todo: default color
            let Paint::Solid(color) = paint;
            styles.push(ir::PathStyle::Stroke(color.to_css().into()));
        }

        SvgItem::Path(ir::PathItem { d, styles })
    }
}
