use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use comemo::Prehashed;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ttf_parser::{GlyphId, OutlineBuilder};
use typst::{
    foundations::Smart,
    introspection::{Introspector, Meta},
    layout::{Abs, Axes, Dir, Frame, FrameItem, FrameKind, Position, Ratio as TypstRatio, Size},
    model::{Destination, Document as TypstDocument},
    text::Font,
    visualize::{
        FixedStroke, Geometry, Gradient, Image as TypstImage, LineCap, LineJoin, Paint, PathItem,
        Pattern, RelativeTo, Shape,
    },
};
use typst::{syntax::Span, text::TextItem as TypstTextItem};

use crate::{
    hash::{Fingerprint, FingerprintBuilder},
    vector::{
        flat_ir,
        ir::{
            self, AbsoluteRef, BuildGlyph, ColorItem, DefId, FontItem, FontRef, GlyphItem,
            GlyphPackBuilderImpl, GlyphRef, GradientKind, Scalar, TransformItem,
        },
        path2d::SvgPath2DBuilder,
        span_id_to_u64,
        utils::{AbsExt, ToCssExt},
    },
    ImmutStr, TakeAs, TypstAbs,
};

use super::{
    FlatPatternItem, FlatSvgItem, FontPack, GlyphPack, GroupRef, ItemPack, LayoutRegion, Page,
    SourceMappingNode,
};

pub type ItemMap = BTreeMap<Fingerprint, FlatSvgItem>;

pub type RefItemMap = HashMap<Fingerprint, (u64, FlatSvgItem)>;
pub type RefItemMapSync = crate::adt::CHashMap<Fingerprint, (u64, FlatSvgItem)>;

pub trait ToItemMap {
    fn to_item_map(self) -> ItemMap;
}

impl ToItemMap for RefItemMap {
    fn to_item_map(self) -> ItemMap {
        self.into_iter().map(|(k, (_, v))| (k, v)).collect::<_>()
    }
}

impl ToItemMap for RefItemMapSync {
    fn to_item_map(self) -> ItemMap {
        self.into_iter().map(|(k, (_, v))| (k, v)).collect::<_>()
    }
}

/// Trait of a streaming representation of a module.
pub trait ModuleStream {
    fn items(&self) -> ItemPack;
    fn layouts(&self) -> Arc<Vec<LayoutRegion>>;
    fn fonts(&self) -> Arc<FontPack>;
    fn glyphs(&self) -> Arc<GlyphPack>;
    fn gc_items(&self) -> Option<Vec<Fingerprint>> {
        // never gc items
        None
    }
}

/// A finished module that stores all the svg items.
/// The svg items shares the underlying data.
/// The svg items are flattened and ready to be serialized.
#[derive(Debug, Default, Clone, Hash)]
pub struct Module {
    pub fonts: Vec<FontItem>,
    pub glyphs: Vec<(DefId, GlyphItem)>,
    pub items: ItemMap,
    pub source_mapping: Vec<SourceMappingNode>,
}

impl Module {
    pub fn freeze(self) -> FrozenModule {
        FrozenModule(Arc::new(Prehashed::new(self)))
    }

    /// Get a font item by its stable ref.
    pub fn get_font(&self, id: &FontRef) -> Option<&FontItem> {
        self.fonts.get(id.idx as usize)
    }

    /// Get a glyph item by its stable ref.
    pub fn get_glyph(&self, id: &AbsoluteRef) -> Option<&GlyphItem> {
        self.glyphs.get(id.id.0 as usize).map(|(_, item)| item)
    }

    /// Get a svg item by its stable ref.
    pub fn get_item(&self, id: &Fingerprint) -> Option<&FlatSvgItem> {
        self.items.get(id)
    }

    pub fn merge_delta(&mut self, v: impl ModuleStream) {
        let item_pack: ItemPack = v.items();
        if let Some(gc_items) = v.gc_items() {
            for id in gc_items {
                self.items.remove(&id);
            }
        }
        self.items.extend(item_pack.0);

        let fonts = v.fonts();
        self.fonts.extend(fonts.take().items);

        let glyphs = v.glyphs();
        self.glyphs.extend(
            glyphs
                .take()
                .items
                .into_iter()
                .map(|(id, item)| (id, item.into())),
        );
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FrozenModule(pub Arc<Prehashed<Module>>);

pub struct ModuleView {
    module: Module,
}

impl ModuleView {
    /// See [`std::path::Path`]
    pub fn new<M: AsRef<Module> + ?Sized>(m: &M) -> &Self {
        unsafe { &*(m.as_ref() as *const Module as *const ModuleView) }
    }
}

impl ToOwned for ModuleView {
    type Owned = Module;

    fn to_owned(&self) -> Self::Owned {
        self.module.clone()
    }
}

impl AsRef<Module> for ModuleView {
    #[inline]
    fn as_ref(&self) -> &Module {
        &self.module
    }
}

impl AsRef<Module> for Module {
    #[inline]
    fn as_ref(&self) -> &Module {
        self
    }
}

impl AsRef<Module> for FrozenModule {
    #[inline]
    fn as_ref(&self) -> &Module {
        self.0.deref().deref()
    }
}

impl AsRef<FrozenModule> for FrozenModule {
    #[inline]
    fn as_ref(&self) -> &FrozenModule {
        self
    }
}

impl Borrow<ModuleView> for FrozenModule {
    fn borrow(&self) -> &ModuleView {
        ModuleView::new(self)
    }
}

impl Borrow<ModuleView> for Module {
    fn borrow(&self) -> &ModuleView {
        ModuleView::new(self)
    }
}

impl Borrow<Module> for FrozenModule {
    fn borrow(&self) -> &Module {
        self.0.deref().deref()
    }
}

/// Intermediate representation of a incompleted svg item.
pub struct ModuleBuilderImpl<const ENABLE_REF_CNT: bool = false> {
    pub glyphs: GlyphPackBuilderImpl<ENABLE_REF_CNT>,
    pub cache_items: crate::adt::CHashMap<Fingerprint, (u64, Fingerprint, FlatSvgItem)>,
    pub items: crate::adt::CHashMap<Fingerprint, (u64, FlatSvgItem)>,
    pub new_items: Mutex<Vec<(Fingerprint, FlatSvgItem)>>,
    pub source_mapping: Vec<SourceMappingNode>,
    pub source_mapping_buffer: Vec<u64>,

    fingerprint_builder: FingerprintBuilder,

    /// See `typst_ts_svg_exporter::ExportFeature`.
    pub should_attach_debug_info: bool,

    pub lifetime: u64,
    pub incr_glyphs: Vec<u64>,
}

pub type ModuleBuilder = ModuleBuilderImpl</* ENABLE_REF_CNT */ false>;
pub type IncrModuleBuilder = ModuleBuilderImpl</* ENABLE_REF_CNT */ true>;

impl<const ENABLE_REF_CNT: bool> Default for ModuleBuilderImpl<ENABLE_REF_CNT> {
    fn default() -> Self {
        Self {
            lifetime: 0,
            cache_items: Default::default(),
            glyphs: Default::default(),
            items: Default::default(),
            new_items: Default::default(),
            source_mapping: Default::default(),
            source_mapping_buffer: Default::default(),
            fingerprint_builder: Default::default(),
            incr_glyphs: Default::default(),
            should_attach_debug_info: false,
        }
    }
}

impl<const ENABLE_REF_CNT: bool> BuildGlyph for ModuleBuilderImpl<ENABLE_REF_CNT> {
    fn build_font(&mut self, font: &Font) -> FontRef {
        self.glyphs.build_font(font)
    }

    fn build_glyph(&mut self, glyph: &GlyphItem) -> GlyphRef {
        self.glyphs.build_glyph(glyph)
    }
}

impl ModuleBuilder {
    pub fn intern(&mut self, module: &Module, f: &Fingerprint) {
        let item = module.get_item(f).unwrap();
        match item {
            FlatSvgItem::None
            | FlatSvgItem::Link(_)
            | FlatSvgItem::Image(_)
            | FlatSvgItem::Path(_)
            | FlatSvgItem::Gradient(_)
            | FlatSvgItem::Pattern(_)
            | FlatSvgItem::ContentHint(_) => {
                self.insert(*f, Cow::Borrowed(item));
            }
            FlatSvgItem::Text(t) => {
                self.glyphs.used_fonts.insert(t.font.clone());
                self.glyphs
                    .used_glyphs
                    .extend(t.content.glyphs.iter().map(|(_, _, glyph)| glyph).cloned());

                self.insert(*f, Cow::Borrowed(item));
            }
            FlatSvgItem::Item(t) => {
                self.insert(*f, Cow::Borrowed(item));

                if !self.items.contains_key(&t.1) {
                    self.intern(module, &t.1);
                }
            }
            FlatSvgItem::Group(g, _) => {
                self.insert(*f, Cow::Borrowed(item));

                for (_, id) in g.0.iter() {
                    if !self.items.contains_key(id) {
                        self.intern(module, id);
                    }
                }
            }
        }
    }
}

impl<const ENABLE_REF_CNT: bool> ModuleBuilderImpl<ENABLE_REF_CNT> {
    pub fn reset(&mut self) {
        self.source_mapping.clear();
        self.source_mapping_buffer.clear();
    }

    fn store_cached<T: Hash>(&self, cond: &T, f: impl FnOnce() -> FlatSvgItem) -> Fingerprint {
        let cond_fg = self.fingerprint_builder.resolve_unchecked(cond);
        self.insert_if(cond_fg, f)
    }

    fn store(&self, item: FlatSvgItem) -> Fingerprint {
        let fingerprint = self.fingerprint_builder.resolve(&item);
        self.insert(fingerprint, Cow::Owned(item));
        fingerprint
    }

    pub fn finalize_ref(&self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
            items: self.items.clone().to_item_map(),
            source_mapping: self.source_mapping.clone(),
        }
    }

    pub fn finalize(self) -> Module {
        let (fonts, glyphs) = self.glyphs.finalize();
        Module {
            fonts,
            glyphs,
            items: self.items.to_item_map(),
            source_mapping: self.source_mapping,
        }
    }

    fn insert_if(&self, cond: Fingerprint, f: impl FnOnce() -> FlatSvgItem) -> Fingerprint {
        if let Some(mut pos) = self.cache_items.get_mut(&cond) {
            if ENABLE_REF_CNT && pos.0 != self.lifetime {
                pos.0 = self.lifetime - 1;
            }

            self.insert(pos.1, Cow::Borrowed(&pos.2));
            return pos.1;
        }

        let item = f();
        let flat_fg = self.fingerprint_builder.resolve(&item);
        self.insert(flat_fg, Cow::Borrowed(&item));

        if ENABLE_REF_CNT {
            self.cache_items
                .insert(cond, (self.lifetime, flat_fg, item));
        } else {
            self.cache_items.insert(cond, (0, flat_fg, item));
        }

        flat_fg
    }

    fn insert(&self, fg: Fingerprint, item: Cow<FlatSvgItem>) -> bool {
        if let Some(mut pos) = self.items.get_mut(&fg) {
            if ENABLE_REF_CNT && pos.0 != self.lifetime {
                pos.0 = self.lifetime - 1;
            }
            return true;
        }

        if ENABLE_REF_CNT {
            self.items.insert(fg, (self.lifetime, FlatSvgItem::None));
            self.new_items.lock().push((fg, item.into_owned()));
        } else {
            self.items.insert(fg, (0, item.into_owned()));
        }

        false
    }

    pub fn build_doc(&self, introspector: &Introspector, doc: &TypstDocument) -> Vec<Page> {
        doc.pages
            .par_iter()
            .map(|p| {
                let abs_ref = self.build(introspector, p);
                Page {
                    content: abs_ref,
                    size: p.size().into(),
                }
            })
            .collect()
    }

    pub fn build(&self, introspector: &Introspector, frame: &Frame) -> Fingerprint {
        // let mut items = Vec::with_capacity(frame.items().len());

        let mut items = frame
            .par_items()
            .flat_map(|(pos, item)| {
                let mut is_link = false;
                let item = match item {
                    FrameItem::Group(group) => {
                        let mut inner = self.build(introspector, &group.frame);

                        if let Some(p) = group.clip_path.as_ref() {
                            // todo: merge
                            let mut builder = SvgPath2DBuilder(String::new());
                            use typst::visualize::PathItem as TypstPathItem;

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

                            inner = self.store(FlatSvgItem::Item(flat_ir::TransformedRef(
                                TransformItem::Clip(Arc::new(ir::PathItem {
                                    d,
                                    size: None,
                                    styles: vec![],
                                })),
                                inner,
                            )));
                        };

                        self.store(FlatSvgItem::Item(flat_ir::TransformedRef(
                            TransformItem::Matrix(Arc::new(group.transform.into())),
                            inner,
                        )))
                    }
                    FrameItem::Text(text) => self.build_text(introspector, text),
                    FrameItem::Shape(shape, s) => self.build_shape(introspector, shape, s),
                    FrameItem::Image(image, size, s) => self.build_image(image, *size, s),
                    FrameItem::Meta(meta, size) => match meta {
                        Meta::Link(lnk) => {
                            is_link = true;
                            self.store(match lnk {
                                Destination::Url(url) => self.lower_link(url, *size),
                                Destination::Position(dest) => self.lower_position(*dest, *size),
                                Destination::Location(loc) => {
                                    // todo: process location before lowering
                                    let dest = introspector.position(*loc);
                                    self.lower_position(dest, *size)
                                }
                            })
                        }
                        // Meta::Link(_) => Fingerprint::from_u128(0),
                        Meta::Elem(elem) => {
                            if !LINE_HINT_ELEMENTS.contains(elem.func().name()) {
                                return None;
                            }

                            self.store(FlatSvgItem::ContentHint('\n'))
                        }
                        #[cfg(not(feature = "no-content-hint"))]
                        Meta::ContentHint(c) => self.store(FlatSvgItem::ContentHint(*c)),
                        // todo: support page label
                        Meta::PdfPageLabel(..) | Meta::PageNumbering(..) | Meta::Hide => {
                            return None
                        }
                    },
                };

                Some(((*pos).into(), is_link, item))
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

        self.store(FlatSvgItem::Group(
            GroupRef(items.into_iter().map(|(x, _, y)| (x, y)).collect()),
            match frame.kind() {
                FrameKind::Hard => Some(frame.size().into()),
                FrameKind::Soft => None,
            },
        ))
    }

    /// Lower a text into svg item.
    pub(super) fn build_text(
        &self,
        introspector: &Introspector,
        text: &TypstTextItem,
    ) -> Fingerprint {
        #[derive(Hash)]
        struct TextHashKey<'i> {
            stateful_fill: Option<Arc<str>>,
            text: &'i TypstTextItem,
        }

        let stateful_fill = match text.fill {
            Paint::Pattern(..) | Paint::Gradient(..) => {
                Some(self.lower_paint(introspector, &text.fill))
            }
            _ => None,
        };

        let cond = TextHashKey {
            stateful_fill: stateful_fill.clone(),
            text,
        };

        self.store_cached(&cond, || {
            let fill = stateful_fill.unwrap_or_else(|| self.lower_paint(introspector, &text.fill));

            let mut glyphs = Vec::with_capacity(text.glyphs.len());
            for glyph in &text.glyphs {
                let id = GlyphId(glyph.id);
                let data = ir::GlyphItem::Raw(text.font.clone(), id);
                let id = self.glyphs.build_glyph(&data);
                // self.glyphs.verify_glyph(id.clone(), &data);
                glyphs.push((
                    glyph.x_offset.at(text.size).into(),
                    glyph.x_advance.at(text.size).into(),
                    id,
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

            let glyphs = glyphs.into();

            let font = self.glyphs.build_font(&text.font);
            FlatSvgItem::Text(flat_ir::FlatTextItem {
                font,
                content: Arc::new(flat_ir::FlatTextItemContent {
                    content: glyph_chars.into(),
                    glyphs,
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
            })
        })
    }

    // /// Lower a geometrical shape into svg item.
    fn build_shape(
        &self,
        introspector: &Introspector,
        shape: &Shape,
        _span_id: &Span,
    ) -> Fingerprint {
        #[derive(Hash)]
        struct ShapeKey<'i> {
            stateful_fill: Option<Arc<str>>,
            stateful_stroke: Option<Arc<str>>,
            shape: &'i Shape,
        }

        let stateful_fill = match shape.fill {
            Some(Paint::Pattern(..) | Paint::Gradient(..)) => {
                Some(self.lower_paint(introspector, shape.fill.as_ref().unwrap()))
            }
            _ => None,
        };

        let stateful_stroke = match shape.stroke {
            Some(FixedStroke {
                paint: Paint::Pattern(..) | Paint::Gradient(..),
                ..
            }) => Some(self.lower_paint(introspector, &shape.stroke.as_ref().unwrap().paint)),
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
                styles.push(ir::PathStyle::Fill(
                    stateful_fill.unwrap_or_else(|| self.lower_paint(introspector, paint_fill)),
                ));
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

                styles.push(ir::PathStyle::Stroke(
                    stateful_stroke.unwrap_or_else(|| self.lower_paint(introspector, paint)),
                ));
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

            FlatSvgItem::Path(item)
        })
    }

    fn build_image(&self, image: &TypstImage, size: Axes<Abs>, _span_id: &Span) -> Fingerprint {
        #[derive(Hash)]
        struct ImageKey<'i> {
            image: &'i TypstImage,
            size: Axes<Abs>,
        }

        let cond = ImageKey { image, size };

        // SvgItem::Image((lower_image(image, *size),
        // span_id_to_u64(span_id)))

        self.store_cached(&cond, || {
            FlatSvgItem::Image(flat_ir::ImageItem {
                image: Arc::new(image.clone().into()),
                size: size.into(),
            })
        })
    }

    // /// Lower a link into svg item.
    pub(super) fn lower_link(&self, url: &str, size: Size) -> FlatSvgItem {
        FlatSvgItem::Link(ir::LinkItem {
            href: url.into(),
            size: size.into(),
        })
    }

    // /// Lower a document position into svg item.
    // #[comemo::memoize]
    pub(super) fn lower_position(&self, pos: Position, size: Size) -> FlatSvgItem {
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

        FlatSvgItem::Link(lnk)
    }
    #[inline]
    pub(super) fn lower_paint(&self, introspector: &Introspector, g: &Paint) -> ImmutStr {
        match g {
            Paint::Solid(c) => c.to_css().into(),
            Paint::Pattern(e) => {
                let fingerprint = self.lower_pattern(introspector, e);
                format!("@{}", fingerprint.as_svg_id("p")).into()
            }
            Paint::Gradient(g) => {
                let fingerprint = self.lower_graident(g);
                format!("@{}", fingerprint.as_svg_id("g")).into()
            }
        }
    }

    pub(super) fn lower_graident(&self, g: &Gradient) -> Fingerprint {
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

        self.store(FlatSvgItem::Gradient(Arc::new(flat_ir::GradientItem {
            stops,
            relative_to_self,
            anti_alias,
            space,
            kind,
            styles,
        })))
    }

    pub(super) fn lower_pattern(&self, introspector: &Introspector, g: &Pattern) -> Fingerprint {
        let frame = self.build(introspector, g.frame());

        let relative_to_self = match g.relative() {
            Smart::Auto => None,
            Smart::Custom(t) => Some(t == RelativeTo::Self_),
        };

        let pattern = FlatSvgItem::Pattern(Arc::new(FlatPatternItem {
            frame,
            size: g.size().into(),
            spacing: g.spacing().into(),
            relative_to_self,
        }));

        self.store(pattern)
    }
}

impl IncrModuleBuilder {
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

    /// Finalize modules containing new svg items.
    pub fn finalize_delta(&mut self) -> Module {
        // filter glyphs by lifetime
        let (fonts, glyphs) = self.glyphs.finalize_delta();

        // filter items by lifetime
        let items = { ItemMap::from_iter(std::mem::take(self.new_items.lock().deref_mut())) };

        Module {
            fonts,
            glyphs,
            items,
            source_mapping: self.source_mapping.clone(),
        }
    }
}

static LINE_HINT_ELEMENTS: once_cell::sync::Lazy<std::collections::HashSet<&'static str>> =
    once_cell::sync::Lazy::new(|| {
        let mut set = std::collections::HashSet::new();
        set.insert("heading");
        set
    });
