use std::{borrow::Cow, cell::RefCell, hash::Hash, ops::DerefMut, sync::Arc};

use parking_lot::Mutex;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ttf_parser::{GlyphId, OutlineBuilder};
use typst::{
    foundations::Smart,
    introspection::{Introspector, Meta},
    layout::{Abs, Axes, Dir, Frame, FrameItem, FrameKind, Position, Ratio as TypstRatio, Size},
    model::{Destination, Document as TypstDocument},
    syntax::Span,
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
        span_id_to_u64,
        utils::{AbsExt, ToCssExt},
    },
    Deferred, ImmutStr, TypstAbs,
};

use super::TGlyph2VecPass;

/// Intermediate representation of a flatten vector item.
pub struct ConvertImpl<const ENABLE_REF_CNT: bool = false> {
    pub glyphs: TGlyph2VecPass<ENABLE_REF_CNT>,
    pub cache_items: crate::adt::CHashMap<Fingerprint, (u64, VecItem)>,
    pub items: crate::adt::CHashMap<Fingerprint, (u64, VecItem)>,
    pub new_items: Mutex<Vec<(Fingerprint, VecItem)>>,

    fingerprint_builder: FingerprintBuilder,

    /// See `typst_ts_svg_exporter::ExportFeature`.
    pub should_attach_debug_info: bool,

    pub lifetime: u64,
    pub incr_glyphs: Vec<u64>,
}

pub type Typst2VecPass = ConvertImpl</* ENABLE_REF_CNT */ false>;
pub type IncrTypst2VecPass = ConvertImpl</* ENABLE_REF_CNT */ true>;

impl<const ENABLE_REF_CNT: bool> Default for ConvertImpl<ENABLE_REF_CNT> {
    fn default() -> Self {
        let glyphs = TGlyph2VecPass::new(GlyphProvider::default(), true);

        Self {
            lifetime: 0,
            cache_items: Default::default(),
            glyphs,
            items: Default::default(),
            new_items: Default::default(),
            fingerprint_builder: Default::default(),
            incr_glyphs: Default::default(),
            should_attach_debug_info: false,
        }
    }
}

impl Typst2VecPass {
    pub fn intern(&mut self, _module: &Module, _f: &Fingerprint) {
        todo!();
        // let item = module.get_item(f).unwrap();
        // match item {
        //     VecItem::None
        //     | VecItem::Link(_)
        //     | VecItem::Image(_)
        //     | VecItem::Path(_)
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
    }
}

impl<const ENABLE_REF_CNT: bool> ConvertImpl<ENABLE_REF_CNT> {
    pub fn reset(&mut self) {}

    pub fn finalize_ref(&self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
            items: self.items.clone().to_item_map(),
        }
    }

    pub fn finalize(self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
            items: self.items.to_item_map(),
        }
    }

    pub fn doc(&self, introspector: &Introspector, doc: &TypstDocument) -> Vec<Page> {
        rayon::scope(move |s| {
            let scope = ConvertScope {
                c: self,
                scope: s,
                cond_defer_list: Mutex::new(vec![]),
            };

            doc.pages.par_iter().for_each(|p| {
                scope.scan_inner(introspector, p);
            });

            let pages = doc
                .pages
                .par_iter()
                .map(|p| {
                    let abs_ref = scope.frame_inner(introspector, p);
                    Page {
                        content: abs_ref,
                        size: p.size().into(),
                    }
                })
                .collect();

            let t = scope.cond_defer_list.into_inner();

            for deferred in t {
                deferred.wait();
            }
            pages
        })
    }

    pub fn frame(&self, introspector: &Introspector, frame: &Frame) -> Fingerprint {
        rayon::scope(move |s| {
            let scope = ConvertScope {
                c: self,
                scope: s,
                cond_defer_list: Mutex::new(vec![]),
            };

            let res = scope.frame_inner(introspector, frame);

            let t = scope.cond_defer_list.into_inner();

            for deferred in t {
                deferred.wait();
            }
            res
        })
    }

    fn insert(&self, fg: Fingerprint, item: Cow<VecItem>) -> bool {
        if let Some(mut pos) = self.items.get_mut(&fg) {
            if ENABLE_REF_CNT && pos.0 != self.lifetime {
                pos.0 = self.lifetime - 1;
            }
            return true;
        }

        if ENABLE_REF_CNT {
            self.items.insert(fg, (self.lifetime, VecItem::None));
            self.new_items.lock().push((fg, item.into_owned()));
        } else {
            self.items.insert(fg, (0, item.into_owned()));
        }

        false
    }
}

struct ConvertScope<'scope, 'a: 'scope, 'b, const ENABLE_REF_CNT: bool> {
    c: &'a ConvertImpl<ENABLE_REF_CNT>,

    scope: &'b rayon::Scope<'scope>,
    cond_defer_list: Mutex<Vec<Deferred<()>>>,
}

impl<'scope, 'a: 'scope, 'b, const ENABLE_REF_CNT: bool>
    ConvertScope<'scope, 'a, 'b, ENABLE_REF_CNT>
{
    fn scan_paint(&self, introspector: &'scope Introspector, g: &'scope Paint) {
        match g {
            Paint::Solid(_) => {}
            Paint::Pattern(e) => {
                self.pattern(introspector, e);
            }
            Paint::Gradient(g) => {
                self.graident(g);
            }
        }
    }

    fn scan_inner(&self, introspector: &'scope Introspector, p: &'scope Frame) {
        p.par_items().for_each(|(_, item)| match item {
            FrameItem::Group(group) => self.scan_inner(introspector, &group.frame),
            FrameItem::Text(text) => {
                self.scan_paint(introspector, &text.fill);

                self.text(introspector, text);
            }
            FrameItem::Shape(shape, span_id) => {
                if let Some(fill) = shape.fill.as_ref() {
                    self.scan_paint(introspector, fill);
                }

                if let Some(stroke) = shape.stroke.as_ref() {
                    self.scan_paint(introspector, &stroke.paint);
                }

                self.shape(introspector, shape, span_id);
            }
            FrameItem::Image(image, s, z) => {
                self.image(image, *s, z);
            }
            FrameItem::Meta(meta, _) => match meta {
                Meta::Link(_) => {}
                Meta::Elem(_) => {}
                Meta::ContentHint(_) => {}
                Meta::PdfPageLabel(..) | Meta::PageNumbering(..) | Meta::Hide => {}
            },
        });
    }

    fn frame_inner(&self, introspector: &'scope Introspector, frame: &'scope Frame) -> Fingerprint {
        let items = frame
            .par_items()
            .flat_map(|(pos, item)| {
                let mut is_link = false;
                let item = match item {
                    FrameItem::Group(group) => {
                        let mut inner = self.frame_inner(introspector, &group.frame);
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

                        self.store(VecItem::Item(TransformedRef(
                            TransformItem::Matrix(Arc::new(group.transform.into())),
                            inner,
                        )))
                    }
                    FrameItem::Text(text) => self.text(introspector, text),
                    FrameItem::Shape(shape, s) => self.shape(introspector, shape, s),
                    FrameItem::Image(image, size, s) => self.image(image, *size, s),
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

                Some(((*pos).into(), is_link, item))
            })
            .collect::<Vec<_>>();

        let mut items = items;

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

        self.store(VecItem::Group(
            GroupRef(items.into_iter().map(|(x, _, y)| (x, y)).collect()),
            match frame.kind() {
                FrameKind::Hard => Some(frame.size().into()),
                FrameKind::Soft => None,
            },
        ))
    }

    fn store_cached<T: Hash>(
        &self,
        cond: &T,
        f: impl FnOnce() -> VecItem + Send + Sync + 'scope + 'a,
    ) -> Fingerprint {
        let cond_fg = self.c.fingerprint_builder.resolve_unchecked(cond);
        self.insert_if(cond_fg, f)
    }

    fn store(&self, item: VecItem) -> Fingerprint {
        let fingerprint = self.c.fingerprint_builder.resolve(&item);
        self.c.insert(fingerprint, Cow::Owned(item));
        fingerprint
    }

    fn insert_if(
        &self,
        cond: Fingerprint,
        f: impl FnOnce() -> VecItem + Send + Sync + 'scope + 'a,
    ) -> Fingerprint {
        let c = self.c;

        if let Some(mut pos) = self.c.cache_items.get_mut(&cond) {
            if ENABLE_REF_CNT && pos.0 != self.c.lifetime {
                pos.0 = self.c.lifetime - 1;
            }

            self.c.insert(cond, Cow::Borrowed(&pos.1));
            return cond;
        }

        self.cond_defer_list
            .lock()
            .push(Deferred::new_in(self.scope, move || {
                let item = f();
                c.insert(cond, Cow::Borrowed(&item));

                if ENABLE_REF_CNT {
                    c.cache_items.insert(cond, (c.lifetime, item));
                } else {
                    c.cache_items.insert(cond, (0, item));
                }
            }));

        cond
    }

    /// Convert a text into vector item.
    fn text(&self, introspector: &'scope Introspector, text: &'scope TypstTextItem) -> Fingerprint {
        let c = self.c;

        let fill = self.paint(introspector, &text.fill);

        self.store_cached(&text, move || {
            let font = c.glyphs.build_font(&text.font);

            let mut glyphs = Vec::with_capacity(text.glyphs.len());
            for glyph in &text.glyphs {
                c.glyphs
                    .build_glyph(font, GlyphItem::Raw(text.font.clone(), GlyphId(glyph.id)));
                glyphs.push((
                    glyph.x_offset.at(text.size).into(),
                    glyph.x_advance.at(text.size).into(),
                    glyph.id as u32,
                ));
            }

            let glyph_chars: String = text.text.to_string();
            // let mut extras = ExtraSvgItems::default();

            let _span_id = text
                .glyphs
                .iter()
                .filter(|g| g.span.0 != Span::detached())
                .map(|g| &g.span.0)
                .map(span_id_to_u64)
                .max()
                .unwrap_or_else(|| span_id_to_u64(&Span::detached()));

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
                    fill,
                    stroke: None,
                }),
            })
        })
    }

    // /// Convert a geometrical shape into vector item.
    fn shape(
        &self,
        introspector: &'scope Introspector,
        shape: &'scope Shape,
        _span_id: &'scope Span,
    ) -> Fingerprint {
        let fill = shape
            .fill
            .as_ref()
            .map(|fill| self.paint(introspector, fill));

        let stroke_paint = shape
            .stroke
            .as_ref()
            .map(|stroke| self.paint(introspector, &stroke.paint));

        self.store_cached(&shape, move || {
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

            if let Some(paint_fill) = fill {
                styles.push(PathStyle::Fill(paint_fill));
            }

            // todo: default miter_limit, thickness
            if let Some(FixedStroke {
                paint: _paint,
                thickness,
                cap,
                join,
                dash,
                miter_limit,
            }) = &shape.stroke
            {
                if let Some(pattern) = dash.as_ref() {
                    styles.push(PathStyle::StrokeDashOffset(pattern.phase.into()));
                    let d = pattern.array.clone();
                    let d = d.into_iter().map(Scalar::from).collect();
                    styles.push(PathStyle::StrokeDashArray(d));
                }

                styles.push(PathStyle::StrokeWidth((*thickness).into()));
                styles.push(PathStyle::StrokeMitterLimit((*miter_limit).into()));
                match cap {
                    LineCap::Butt => {}
                    LineCap::Round => styles.push(PathStyle::StrokeLineCap("round".into())),
                    LineCap::Square => styles.push(PathStyle::StrokeLineCap("square".into())),
                };
                match join {
                    LineJoin::Miter => {}
                    LineJoin::Bevel => styles.push(PathStyle::StrokeLineJoin("bevel".into())),
                    LineJoin::Round => styles.push(PathStyle::StrokeLineJoin("round".into())),
                }

                styles.push(PathStyle::Stroke(stroke_paint.unwrap()));
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
                size: Some(shape_size.into()),
                styles,
            };

            VecItem::Path(item)
        })
    }

    fn image(
        &self,
        image: &'scope TypstImage,
        size: Axes<Abs>,
        _span_id: &'scope Span,
    ) -> Fingerprint {
        #[derive(Hash)]
        struct ImageKey<'i> {
            image: &'i TypstImage,
            size: Axes<Abs>,
        }

        let cond = ImageKey { image, size };

        // SvgItem::Image((image(image, *size), span_id_to_u64(span_id)))

        self.store_cached(&cond, move || {
            VecItem::Image(ImageItem {
                image: Arc::new(image.clone().into()),
                size: size.into(),
            })
        })
    }

    // /// Convert a link into vector item.
    fn link(&self, url: &str, size: Size) -> VecItem {
        VecItem::Link(LinkItem {
            href: url.into(),
            size: size.into(),
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
            size: size.into(),
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
                    styles.push(GradientStyle::Center(l.center.into()));
                }

                if l.focal_center.x != TypstRatio::new(0.5)
                    || l.focal_center.y != TypstRatio::new(0.5)
                {
                    styles.push(GradientStyle::FocalCenter(l.focal_center.into()));
                }

                if l.focal_radius != TypstRatio::zero() {
                    styles.push(GradientStyle::FocalRadius(l.focal_radius.into()));
                }

                GradientKind::Radial(l.radius.into())
            }
            Gradient::Conic(l) => {
                if l.center.x != TypstRatio::new(0.5) || l.center.y != TypstRatio::new(0.5) {
                    styles.push(GradientStyle::Center(l.center.into()));
                }

                GradientKind::Conic(l.angle.into())
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
        let frame = self.c.frame(introspector, g.frame());

        let relative_to_self = match g.relative() {
            Smart::Auto => None,
            Smart::Custom(t) => Some(t == RelativeTo::Self_),
        };

        let pattern = VecItem::Pattern(Arc::new(PatternItem {
            frame,
            size: g.size().into(),
            spacing: g.spacing().into(),
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
        let gc_items = RefCell::new(vec![]);

        // a threshold is set by current lifetime subtracted by the given threshold.
        // It uses saturating_sub to prevent underflow (u64).
        let gc_threshold = self.lifetime.saturating_sub(threshold);

        self.items.retain(|k, v| {
            if v.0 < gc_threshold {
                gc_items.borrow_mut().push(*k);
                false
            } else {
                true
            }
        });

        // Same as above
        let cache_threshold = self.lifetime.saturating_sub(threshold);
        self.cache_items.retain(|_, v| v.0 >= cache_threshold);

        gc_items.into_inner()
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

static LINE_HINT_ELEMENTS: once_cell::sync::Lazy<std::collections::HashSet<&'static str>> =
    once_cell::sync::Lazy::new(|| {
        let mut set = std::collections::HashSet::new();
        set.insert("heading");
        set
    });
