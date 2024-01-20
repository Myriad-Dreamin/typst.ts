use std::{
    borrow::Cow,
    hash::Hash,
    ops::DerefMut,
    sync::{atomic::AtomicU64, Arc},
};

use parking_lot::Mutex;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use ttf_parser::{GlyphId, OutlineBuilder};
use typst::{
    foundations::Smart,
    introspection::{Introspector, Meta},
    layout::{
        Abs, Axes, Dir, Frame, FrameItem, FrameKind, Position, Ratio as TypstRatio, Size,
        Transform as TypstTransform,
    },
    model::{Destination, Document as TypstDocument},
    text::TextItem as TypstTextItem,
    visualize::{
        FixedStroke, Geometry, Gradient, Image as TypstImage, LineCap, LineJoin, Paint,
        PathItem as TypstPathItem, Pattern, RelativeTo, Shape,
    },
};

use crate::{
    font::GlyphProvider,
    hash::{Fingerprint, FingerprintBuilder},
    vector::{
        ir::*,
        path2d::SvgPath2DBuilder,
        utils::{AbsExt, ToCssExt},
    },
    FromTypst, ImmutStr, IntoTypst, TypstAbs,
};

use super::{SourceNodeKind, SourceRegion, Span2VecPass, TGlyph2VecPass};

/// Intermediate representation of a flatten vector item.
pub struct ConvertImpl<const ENABLE_REF_CNT: bool = false> {
    pub glyphs: TGlyph2VecPass<ENABLE_REF_CNT>,
    pub spans: Span2VecPass,
    pub cache_items: RefItemMapT<(AtomicU64, Fingerprint, VecItem)>,
    pub items: RefItemMapSync,
    pub new_items: Mutex<Vec<(Fingerprint, VecItem)>>,

    fingerprint_builder: FingerprintBuilder,

    pub lifetime: u64,
}

pub type Typst2VecPass = ConvertImpl</* ENABLE_REF_CNT */ false>;
pub type IncrTypst2VecPass = ConvertImpl</* ENABLE_REF_CNT */ true>;

impl<const ENABLE_REF_CNT: bool> Default for ConvertImpl<ENABLE_REF_CNT> {
    fn default() -> Self {
        let glyphs = TGlyph2VecPass::new(GlyphProvider::default(), true);
        let spans = Span2VecPass::default();

        Self {
            lifetime: 0,
            glyphs,
            spans,
            cache_items: Default::default(),
            items: Default::default(),
            new_items: Default::default(),
            fingerprint_builder: Default::default(),
        }
    }
}

impl Typst2VecPass {
    pub fn intern(&mut self, _module: &Module, _f: &Fingerprint) {
        // let item = module.get_item(f).unwrap();
        // match item {
        //     VecItem::None
        //     | VecItem::Link(_)
        //     | VecItem::Image(_)
        //     | VecItem::Path(_)
        //     | VecItem::Color32(_)
        //     | VecItem::Gradient(_)
        //     | VecItem::Pattern(_)
        //     | VecItem::ContentHint(_) => {
        //         self.insert(*f, Cow::Borrowed(item));
        //     }
        //     VecItem::Text(_t) => {
        //         // self.glyphs.used_fonts.insert(t.shape.font.clone());
        //         // self.glyphs
        //         //     .used_glyphs
        //         //     .extend(t.content.glyphs.iter().map(|(_, _, glyph)|
        // glyph).cloned());

        //         // self.insert(*f, Cow::Borrowed(item));
        //         todo!()
        //     }
        //     VecItem::Item(t) => {
        //         self.insert(*f, Cow::Borrowed(item));

        //         if !self.items.contains_key(&t.1) {
        //             self.intern(module, &t.1);
        //         }
        //     }
        //     VecItem::Group(g, _) => {
        //         self.insert(*f, Cow::Borrowed(item));

        //         for (_, id) in g.0.iter() {
        //             if !self.items.contains_key(id) {
        //                 self.intern(module, id);
        //             }
        //         }
        //     }
        // }
        todo!()
    }
}

impl<const ENABLE_REF_CNT: bool> ConvertImpl<ENABLE_REF_CNT> {
    pub fn reset(&mut self) {}

    pub fn finalize(self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
            items: self.items.to_item_map(),
        }
    }

    pub fn doc(&self, introspector: &Introspector, doc: &TypstDocument) -> Vec<Page> {
        let doc_reg = self.spans.start();

        let pages = doc
            .pages
            .par_iter()
            .enumerate()
            .map(|(idx, p)| {
                let page_reg = self.spans.start();

                let abs_ref = self.frame(introspector, p, page_reg, idx);

                self.spans.push_span(SourceRegion {
                    region: doc_reg,
                    idx: idx as u32,
                    kind: SourceNodeKind::Page { region: page_reg },
                    item: abs_ref,
                });

                Page {
                    content: abs_ref,
                    size: p.size().into_typst(),
                }
            })
            .collect();

        self.spans
            .doc_region
            .store(doc_reg, std::sync::atomic::Ordering::SeqCst);

        pages
    }

    pub fn frame(
        &self,
        introspector: &Introspector,
        frame: &Frame,
        parent_reg: usize,
        index: usize,
    ) -> Fingerprint {
        // let mut items = Vec::with_capacity(frame.items().len());
        let src_reg = self.spans.start();

        let mut items = frame
            .items()
            .as_slice()
            .par_iter()
            .enumerate()
            .flat_map(|(idx, (pos, item))| {
                let mut is_link = false;
                let item = match item {
                    FrameItem::Group(group) => {
                        let mut inner = self.frame(introspector, &group.frame, src_reg, idx);

                        if let Some(p) = group.clip_path.as_ref() {
                            // todo: merge
                            let mut builder = SvgPath2DBuilder(String::new());

                            // to ensure that our shape focus on the original point
                            builder.move_to(0., 0.);
                            for elem in &p.0 {
                                match elem {
                                    TypstPathItem::MoveTo(p) => {
                                        builder.move_to(p.x.to_f32(), p.y.to_f32());
                                    }
                                    TypstPathItem::LineTo(p) => {
                                        builder.line_to(p.x.to_f32(), p.y.to_f32());
                                    }
                                    TypstPathItem::CubicTo(p1, p2, p3) => {
                                        builder.curve_to(
                                            p1.x.to_f32(),
                                            p1.y.to_f32(),
                                            p2.x.to_f32(),
                                            p2.y.to_f32(),
                                            p3.x.to_f32(),
                                            p3.y.to_f32(),
                                        );
                                    }
                                    TypstPathItem::ClosePath => {
                                        builder.close();
                                    }
                                };
                            }
                            let d = builder.0.into();

                            inner = self.store(VecItem::Item(TransformedRef(
                                TransformItem::Clip(Arc::new(PathItem {
                                    d,
                                    size: None,
                                    styles: vec![],
                                })),
                                inner,
                            )));
                        };

                        if group.transform != TypstTransform::identity() {
                            inner = self.store(VecItem::Item(TransformedRef(
                                TransformItem::Matrix(Arc::new(group.transform.into_typst())),
                                inner,
                            )));
                        }

                        inner
                    }
                    FrameItem::Text(text) => {
                        let i = self.text(introspector, text);

                        self.spans.push_span(SourceRegion {
                            region: src_reg,
                            idx: idx as u32,
                            kind: if text.glyphs.len() == 1 {
                                SourceNodeKind::Char(text.glyphs[0].span)
                            } else {
                                SourceNodeKind::Text(text.glyphs.iter().map(|g| g.span).collect())
                            },
                            item: i,
                        });

                        i
                    }
                    FrameItem::Shape(shape, s) => {
                        let i = self.shape(introspector, shape);

                        self.spans.push_span(SourceRegion {
                            region: src_reg,
                            idx: idx as u32,
                            kind: SourceNodeKind::Shape(*s),
                            item: i,
                        });

                        i
                    }
                    FrameItem::Image(image, size, s) => {
                        let i = self.image(image, *size);

                        self.spans.push_span(SourceRegion {
                            region: src_reg,
                            idx: idx as u32,
                            kind: SourceNodeKind::Image(*s),
                            item: i,
                        });

                        i
                    }
                    FrameItem::Meta(meta, size) => match meta {
                        Meta::Link(lnk) => {
                            is_link = true;
                            self.store(match lnk {
                                Destination::Url(url) => self.link(url, *size),
                                Destination::Position(dest) => self.position(*dest, *size),
                                Destination::Location(loc) => {
                                    // todo: process location before lowering
                                    let dest = introspector.position(*loc);
                                    self.position(dest, *size)
                                }
                            })
                        }
                        // Meta::Link(_) => Fingerprint::from_u128(0),
                        Meta::Elem(elem) => {
                            if !LINE_HINT_ELEMENTS.contains(elem.func().name()) {
                                return None;
                            }

                            self.store(VecItem::ContentHint('\n'))
                        }
                        #[cfg(not(feature = "no-content-hint"))]
                        Meta::ContentHint(c) => self.store(VecItem::ContentHint(*c)),
                        // todo: support page label
                        Meta::PdfPageLabel(..) | Meta::PageNumbering(..) | Meta::Hide => {
                            return None
                        }
                    },
                };

                Some(((*pos).into_typst(), is_link, item))
            })
            .collect::<Vec<_>>();

        // swap link items
        items.sort_by(|x, y| {
            let x_is_link = x.1;
            let y_is_link = y.1;
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

        let g = self.store(VecItem::Group(
            GroupRef(items.into_iter().map(|(x, _, y)| (x, y)).collect()),
            match frame.kind() {
                FrameKind::Hard => Some(frame.size().into_typst()),
                FrameKind::Soft => None,
            },
        ));

        self.spans.push_span(SourceRegion {
            region: parent_reg,
            idx: index as u32,
            kind: SourceNodeKind::Group { region: src_reg },
            item: g,
        });

        g
    }

    fn store_cached<T: Hash>(&self, cond: &T, f: impl FnOnce() -> VecItem) -> Fingerprint {
        let cond_fg = self.fingerprint_builder.resolve_unchecked(cond);
        self.insert_if(cond_fg, f)
    }

    fn store(&self, item: VecItem) -> Fingerprint {
        let fingerprint = self.fingerprint_builder.resolve(&item);
        self.insert(fingerprint, Cow::Owned(item));
        fingerprint
    }

    /// Increases the lifetime of an item.
    ///
    /// Note: See [`Self::increment_lifetime`], the `self.lifetime` increases by
    /// 2 each time.
    fn increase_lifetime_for_item(&self, pos: &AtomicU64) {
        let c = pos.load(std::sync::atomic::Ordering::Relaxed);
        if ENABLE_REF_CNT && c < self.lifetime - 1 {
            // Note that the Vec2Item is locked by mutable reference. And during update,
            // lifetime will be updated to either self.lifetime or self.lifetime
            // - 1. This indicates that it is fine to ignore the result of compare_exchange.
            //
            // If compare_exchange fails, it means that it is updated to self.lifetime
            // Otherwise, it is updated to self.lifetime - 1
            //
            // Both cases are fine, as we renew the lifetime of the item.
            let _ = pos.compare_exchange(
                c,
                self.lifetime - 1,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            );
        }
    }

    fn insert_if(&self, cond: Fingerprint, f: impl FnOnce() -> VecItem) -> Fingerprint {
        let shard = &self.cache_items.shard(cond);
        let shard_read = shard.read();
        if let Some(pos) = shard_read.get(&cond) {
            self.increase_lifetime_for_item(&pos.0);
            self.insert(pos.1, Cow::Borrowed(&pos.2));
            return pos.1;
        }

        drop(shard_read);

        let item = f();
        let flat_fg = self.fingerprint_builder.resolve(&item);
        self.insert(flat_fg, Cow::Borrowed(&item));

        {
            let mut shard_write = shard.write();
            shard_write.insert(
                cond,
                if ENABLE_REF_CNT {
                    (AtomicU64::new(self.lifetime), flat_fg, item)
                } else {
                    (AtomicU64::new(0), flat_fg, item)
                },
            );
        }

        flat_fg
    }

    fn insert(&self, fg: Fingerprint, item: Cow<VecItem>) -> bool {
        let shard = self.items.shard(fg);
        let shard_read = shard.read();
        if let Some(pos) = shard_read.get(&fg) {
            self.increase_lifetime_for_item(&pos.0);
            return true;
        }

        let item_resolution = if ENABLE_REF_CNT {
            self.new_items.lock().push((fg, item.into_owned()));
            (AtomicU64::new(self.lifetime), VecItem::None)
        } else {
            (AtomicU64::new(0), item.into_owned())
        };

        drop(shard_read);
        let mut shard_write = shard.write();
        shard_write.insert(fg, item_resolution);
        false
    }

    #[cfg(feature = "item-dashmap")]
    fn insert_if(&self, cond: Fingerprint, f: impl FnOnce() -> VecItem) -> Fingerprint {
        use dashmap::mapref::entry::Entry::*;
        match self.cache_items.entry(cond) {
            Occupied(pos) => {
                let pos = pos.into_ref();
                self.increase_lifetime(&pos.0);
                self.insert(pos.1, Cow::Borrowed(&pos.2));
                pos.1
            }
            Vacant(pos) => {
                let item = f();
                let flat_fg = self.fingerprint_builder.resolve(&item);
                self.insert(flat_fg, Cow::Borrowed(&item));

                pos.insert(if ENABLE_REF_CNT {
                    (AtomicU64::new(self.lifetime), flat_fg, item)
                } else {
                    (AtomicU64::new(0), flat_fg, item)
                });

                flat_fg
            }
        }
    }

    #[cfg(feature = "item-dashmap")]
    fn insert(&self, fg: Fingerprint, item: Cow<VecItem>) -> bool {
        use dashmap::mapref::entry::Entry::*;
        match self.items.entry(fg) {
            Occupied(pos) => {
                let pos = pos.into_ref();
                self.increase_lifetime(&pos.0);
                true
            }
            Vacant(pos) => {
                let item_resolution = if ENABLE_REF_CNT {
                    self.new_items.lock().push((fg, item.into_owned()));
                    (AtomicU64::new(self.lifetime), VecItem::None)
                } else {
                    (AtomicU64::new(0), item.into_owned())
                };

                pos.insert(item_resolution);
                false
            }
        }
    }

    /// Convert a text into vector item.
    pub fn text(&self, introspector: &Introspector, text: &TypstTextItem) -> Fingerprint {
        let stateful_fill = match text.fill {
            Paint::Pattern(..) | Paint::Gradient(..) => Some(self.paint(introspector, &text.fill)),
            _ => None,
        };

        #[cfg(feature = "text-stroke")]
        let stateful_stroke = match &text.stroke {
            Some(FixedStroke {
                paint: Paint::Pattern(..) | Paint::Gradient(..),
                ..
            }) => Some(self.paint(introspector, &text.stroke.as_ref().unwrap().paint)),
            _ => None,
        };

        #[derive(Hash)]
        struct TextHashKey<'i> {
            stateful_fill: Option<Arc<str>>,
            #[cfg(feature = "text-stroke")]
            stateful_stroke: Option<Arc<str>>,
            text: &'i TypstTextItem,
        }

        let cond = TextHashKey {
            stateful_fill: stateful_fill.clone(),
            #[cfg(feature = "text-stroke")]
            stateful_stroke: stateful_stroke.clone(),
            text,
        };

        self.store_cached(&cond, || {
            let font = self.glyphs.build_font(&text.font);

            let mut glyphs = Vec::with_capacity(text.glyphs.len());
            for glyph in &text.glyphs {
                self.glyphs
                    .build_glyph(font, GlyphItem::Raw(text.font.clone(), GlyphId(glyph.id)));
                glyphs.push((
                    glyph.x_offset.at(text.size).into_typst(),
                    glyph.x_advance.at(text.size).into_typst(),
                    glyph.id as u32,
                ));
            }

            let glyph_chars: String = text.text.to_string();
            // let mut extras = ExtraSvgItems::default();

            let font = self.glyphs.build_font(&text.font);
            let fill = stateful_fill.unwrap_or_else(|| self.paint(introspector, &text.fill));

            let styles = vec![PathStyle::Fill(fill)];
            #[cfg(feature = "text-stroke")]
            if let Some(stroke) = text.stroke.as_ref() {
                self.stroke(introspector, stateful_stroke, stroke, &mut styles);
            }

            VecItem::Text(TextItem {
                content: Arc::new(TextItemContent {
                    content: glyph_chars.into(),
                    glyphs: glyphs.into(),
                }),
                shape: Arc::new(TextShape {
                    font,
                    size: Scalar(text.size.to_f32()),
                    dir: match text.lang.dir() {
                        Dir::LTR => "ltr",
                        Dir::RTL => "rtl",
                        Dir::TTB => "ttb",
                        Dir::BTT => "btt",
                    }
                    .into(),
                    styles,
                }),
            })
        })
    }

    fn stroke(
        &self,
        introspector: &Introspector,
        stateful_stroke: Option<Arc<str>>,
        FixedStroke {
            paint,
            thickness,
            line_cap,
            line_join,
            dash_pattern,
            miter_limit,
        }: &FixedStroke,
        styles: &mut Vec<PathStyle>,
    ) {
        // todo: default miter_limit, thickness
        if let Some(pattern) = dash_pattern.as_ref() {
            styles.push(PathStyle::StrokeDashOffset(pattern.phase.into_typst()));
            let d = pattern.array.clone();
            let d = d.into_iter().map(Scalar::from_typst).collect();
            styles.push(PathStyle::StrokeDashArray(d));
        }

        styles.push(PathStyle::StrokeWidth((*thickness).into_typst()));
        styles.push(PathStyle::StrokeMitterLimit((*miter_limit).into_typst()));
        match line_cap {
            LineCap::Butt => {}
            LineCap::Round => styles.push(PathStyle::StrokeLineCap("round".into())),
            LineCap::Square => styles.push(PathStyle::StrokeLineCap("square".into())),
        };
        match line_join {
            LineJoin::Miter => {}
            LineJoin::Bevel => styles.push(PathStyle::StrokeLineJoin("bevel".into())),
            LineJoin::Round => styles.push(PathStyle::StrokeLineJoin("round".into())),
        }

        styles.push(PathStyle::Stroke(
            stateful_stroke.unwrap_or_else(|| self.paint(introspector, paint)),
        ));
    }

    // /// Convert a geometrical shape into vector item.
    pub fn shape(&self, introspector: &Introspector, shape: &Shape) -> Fingerprint {
        #[derive(Hash)]
        struct ShapeKey<'i> {
            stateful_fill: Option<Arc<str>>,
            stateful_stroke: Option<Arc<str>>,
            shape: &'i Shape,
        }

        let stateful_fill = match shape.fill {
            Some(Paint::Pattern(..) | Paint::Gradient(..)) => {
                Some(self.paint(introspector, shape.fill.as_ref().unwrap()))
            }
            _ => None,
        };

        let stateful_stroke = match shape.stroke {
            Some(FixedStroke {
                paint: Paint::Pattern(..) | Paint::Gradient(..),
                ..
            }) => Some(self.paint(introspector, &shape.stroke.as_ref().unwrap().paint)),
            _ => None,
        };

        let cond = &ShapeKey {
            stateful_fill: stateful_fill.clone(),
            stateful_stroke: stateful_stroke.clone(),
            shape,
        };

        self.store_cached(cond, || {
            let mut builder = SvgPath2DBuilder(String::new());
            // let mut extras = ExtraSvgItems::default();

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
                            TypstPathItem::MoveTo(p) => {
                                builder.move_to(p.x.to_f32(), p.y.to_f32());
                            }
                            TypstPathItem::LineTo(p) => {
                                builder.line_to(p.x.to_f32(), p.y.to_f32());
                            }
                            TypstPathItem::CubicTo(p1, p2, p3) => {
                                builder.curve_to(
                                    p1.x.to_f32(),
                                    p1.y.to_f32(),
                                    p2.x.to_f32(),
                                    p2.y.to_f32(),
                                    p3.x.to_f32(),
                                    p3.y.to_f32(),
                                );
                            }
                            TypstPathItem::ClosePath => {
                                builder.close();
                            }
                        };
                    }
                }
            };

            let d = builder.0.into();

            let mut styles = Vec::new();

            if let Some(paint_fill) = &shape.fill {
                styles.push(PathStyle::Fill(
                    stateful_fill.unwrap_or_else(|| self.paint(introspector, paint_fill)),
                ));
            }

            if let Some(stroke) = &shape.stroke {
                self.stroke(introspector, stateful_stroke, stroke, &mut styles);
            }

            let mut shape_size = shape.geometry.bbox_size();
            // Edge cases for strokes.
            if shape_size.x.to_pt() == 0.0 {
                shape_size.x = TypstAbs::pt(1.0);
            }

            if shape_size.y.to_pt() == 0.0 {
                shape_size.y = TypstAbs::pt(1.0);
            }

            let item = PathItem {
                d,
                size: Some(shape_size.into_typst()),
                styles,
            };

            VecItem::Path(item)
        })
    }

    pub fn image(&self, image: &TypstImage, size: Axes<Abs>) -> Fingerprint {
        #[derive(Hash)]
        struct ImageKey<'i> {
            image: &'i TypstImage,
            size: Axes<Abs>,
        }

        let cond = ImageKey { image, size };

        self.store_cached(&cond, || {
            VecItem::Image(ImageItem {
                image: Arc::new(image.clone().into_typst()),
                size: size.into_typst(),
            })
        })
    }

    // /// Convert a link into vector item.
    fn link(&self, url: &str, size: Size) -> VecItem {
        VecItem::Link(LinkItem {
            href: url.into(),
            size: size.into_typst(),
        })
    }

    // /// Convert a document position into vector item.
    // #[comemo::memoize]
    fn position(&self, pos: Position, size: Size) -> VecItem {
        let lnk = LinkItem {
            href: format!(
                "@typst:handleTypstLocation(this, {}, {}, {})",
                pos.page,
                pos.point.x.to_f32(),
                pos.point.y.to_f32()
            )
            .into(),
            size: size.into_typst(),
        };

        VecItem::Link(lnk)
    }
    #[inline]
    fn paint(&self, introspector: &Introspector, g: &Paint) -> ImmutStr {
        match g {
            Paint::Solid(c) => c.to_css().into(),
            Paint::Pattern(e) => {
                let fingerprint = self.pattern(introspector, e);
                format!("@{}", fingerprint.as_svg_id("p")).into()
            }
            Paint::Gradient(g) => {
                let fingerprint = self.graident(g);
                format!("@{}", fingerprint.as_svg_id("g")).into()
            }
        }
    }

    fn graident(&self, g: &Gradient) -> Fingerprint {
        let mut stops = Vec::with_capacity(g.stops_ref().len());
        for (c, step) in g.stops_ref() {
            let [r, g, b, a] = c.to_vec4_u8();
            stops.push((Rgba8Item { r, g, b, a }, (*step).into_typst()))
        }

        let relative_to_self = match g.relative() {
            Smart::Auto => None,
            Smart::Custom(t) => Some(t == RelativeTo::Self_),
        };

        let anti_alias = g.anti_alias();
        let space = g.space().into_typst();

        let mut styles = Vec::new();
        let kind = match g {
            Gradient::Linear(l) => GradientKind::Linear(l.angle.into_typst()),
            Gradient::Radial(l) => {
                if l.center.x != TypstRatio::new(0.5) || l.center.y != TypstRatio::new(0.5) {
                    styles.push(GradientStyle::Center(l.center.into_typst()));
                }

                if l.focal_center.x != TypstRatio::new(0.5)
                    || l.focal_center.y != TypstRatio::new(0.5)
                {
                    styles.push(GradientStyle::FocalCenter(l.focal_center.into_typst()));
                }

                if l.focal_radius != TypstRatio::zero() {
                    styles.push(GradientStyle::FocalRadius(l.focal_radius.into_typst()));
                }

                GradientKind::Radial(l.radius.into_typst())
            }
            Gradient::Conic(l) => {
                if l.center.x != TypstRatio::new(0.5) || l.center.y != TypstRatio::new(0.5) {
                    styles.push(GradientStyle::Center(l.center.into_typst()));
                }

                GradientKind::Conic(l.angle.into_typst())
            }
        };

        self.store(VecItem::Gradient(Arc::new(GradientItem {
            stops,
            relative_to_self,
            anti_alias,
            space,
            kind,
            styles,
        })))
    }

    fn pattern(&self, introspector: &Introspector, g: &Pattern) -> Fingerprint {
        let frame = self.frame(introspector, g.frame(), 0, 0);

        let relative_to_self = match g.relative() {
            Smart::Auto => None,
            Smart::Custom(t) => Some(t == RelativeTo::Self_),
        };

        let pattern = VecItem::Pattern(Arc::new(PatternItem {
            frame,
            size: g.size().into_typst(),
            spacing: g.spacing().into_typst(),
            relative_to_self,
        }));

        self.store(pattern)
    }
}

impl IncrTypst2VecPass {
    /// Increment the lifetime of the module.
    /// It increments by 2 which is used to distinguish between the
    /// retained items and the new items.
    /// Assuming that the old lifetime is 'l,
    /// the retained and new lifetime will be 'l + 1 and 'l + 2, respectively.
    pub fn increment_lifetime(&mut self) {
        self.new_items.get_mut().clear();
        self.glyphs.new_fonts.get_mut().clear();
        self.glyphs.new_glyphs.get_mut().clear();
        self.lifetime += 2;
        self.glyphs.lifetime = self.lifetime;
    }

    /// Perform garbage collection with given threshold.
    pub fn gc(&mut self, threshold: u64) -> Vec<Fingerprint> {
        let gc_items = Arc::new(Mutex::new(vec![]));

        // a threshold is set by current lifetime subtracted by the given threshold.
        // It uses saturating_sub to prevent underflow (u64).
        let gc_threshold = self.lifetime.saturating_sub(threshold);

        self.items.as_mut_slice().par_iter_mut().for_each(|e| {
            e.get_mut().retain(|k, v| {
                if v.0.load(std::sync::atomic::Ordering::Relaxed) < gc_threshold {
                    gc_items.lock().push(*k);
                    false
                } else {
                    true
                }
            });
        });

        // Same as above
        let cache_threshold = self.lifetime.saturating_sub(threshold);
        self.cache_items
            .as_mut_slice()
            .par_iter_mut()
            .for_each(|e| {
                e.get_mut().retain(|_, v| {
                    v.0.load(std::sync::atomic::Ordering::Relaxed) >= cache_threshold
                });
            });

        Arc::try_unwrap(gc_items).unwrap().into_inner()
    }

    /// Finalize modules containing new vector items.
    pub fn finalize_delta(&mut self) -> Module {
        // filter glyphs by lifetime
        let (fonts, glyphs) = self.glyphs.finalize_delta();

        // filter items by lifetime
        let items = { ItemMap::from_iter(std::mem::take(self.new_items.lock().deref_mut())) };

        Module {
            fonts,
            glyphs,
            items,
        }
    }
}

// impl<'m, const ENABLE_REF_CNT: bool> ItemIndice<'m> for
// ConvertImpl<ENABLE_REF_CNT> {     fn get_item(&self, value: &Fingerprint) ->
// Option<&'m VecItem> {         self.items.get(value).map(|item| &item.1)
//     }
// }

static LINE_HINT_ELEMENTS: once_cell::sync::Lazy<std::collections::HashSet<&'static str>> =
    once_cell::sync::Lazy::new(|| {
        let mut set = std::collections::HashSet::new();
        set.insert("heading");
        set
    });
