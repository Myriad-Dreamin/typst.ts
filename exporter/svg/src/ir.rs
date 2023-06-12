use std::any::Any;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use base64::Engine;
use once_cell::sync::Lazy;
use rustc_hash::FxHasher;
use siphasher::sip128::SipHasher13;
use ttf_parser::GlyphId;
use typst::font::Font;
use typst::geom::{Abs, Axes, Dir, Point, Ratio, Scalar, Size, Transform};
use typst::image::Image;
use typst::util::Buffer;

pub type ImmutStr = Arc<str>;

/// The local id of a svg item.
/// This id is only unique within the svg document.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DefId(pub u64);

impl DefId {
    /// Make a [`RelativeDefId`] relative to the given id.
    pub fn make_relative(&self, id: DefId) -> RelativeDefId {
        RelativeDefId(id.0 as i64 - self.0 as i64)
    }

    /// Make a [`DefId`] from the given relative id.
    pub fn make_absolute(&self, id: RelativeDefId) -> DefId {
        DefId((id.0 + self.0 as i64) as u64)
    }

    /// Make a [`RelativeRef`] relative to the given id.
    pub fn make_relative_ref(&self, abs_ref: AbsoulteRef) -> RelativeRef {
        RelativeRef {
            fingerprint: abs_ref.fingerprint,
            id: self.make_relative(abs_ref.id),
        }
    }

    /// Make a [`AbsoulteRef`] from the given relative id.
    pub fn make_absolute_ref(&self, rel_ref: RelativeRef) -> AbsoulteRef {
        AbsoulteRef {
            fingerprint: rel_ref.fingerprint,
            id: self.make_absolute(rel_ref.id),
        }
    }
}

/// The relative id of a svg item.
/// See:
/// + [`DefId::make_relative_ref`]
/// + [`DefId::make_absolute_ref`]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelativeDefId(pub i64);

/// See <https://github.com/rust-lang/rust/blob/master/compiler/rustc_hir/src/stable_hash_impls.rs#L22>
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Fingerprint(u64, u64);

pub trait FingerprintHasher: std::hash::Hasher {
    fn finish_fingerprint(&self) -> (u64, Vec<u8>);
}

/// A stable absolute reference.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AbsoulteRef {
    pub fingerprint: Fingerprint,
    pub id: DefId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeakAbsoulteRef(pub AbsoulteRef);

impl WeakAbsoulteRef {
    pub fn as_svg_id(&self, prefix: &'static str) -> String {
        self.0.as_svg_id(prefix)
    }
}

impl Hash for WeakAbsoulteRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.fingerprint.hash(state);
    }
}

/// A stable relative reference.
/// These objects can only be constructed relative from a [`AbsoulteRef`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RelativeRef {
    pub fingerprint: Fingerprint,
    pub id: RelativeDefId,
}

impl RelativeRef {
    pub(crate) fn as_svg_id(&self, prefix: &'static str) -> String {
        AbsoulteRef::as_svg_id_inner(self.fingerprint, prefix)
    }
}

impl AbsoulteRef {
    /// Create a xml id from the given prefix and the fingerprint of this reference.
    /// Note that the entire html document shares namespace for ids.
    #[comemo::memoize]
    fn as_svg_id_inner(fingerprint: Fingerprint, prefix: &'static str) -> String {
        let fg =
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(fingerprint.0.to_le_bytes());
        if fingerprint.1 == 0 {
            return [prefix, &fg].join("");
        }

        let id = {
            let id = fingerprint.1.to_le_bytes();
            // truncate zero
            let rev_zero = id.iter().rev().skip_while(|&&b| b == 0).count();
            let id = &id[..rev_zero];
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(id)
        };
        [prefix, &fg, &id].join("")
    }

    #[inline]
    pub fn as_svg_id(&self, prefix: &'static str) -> String {
        Self::as_svg_id_inner(self.fingerprint, prefix)
    }
}

/// A finished module that stores all the svg items.
/// The svg items shares the underlying data.
/// The svg items are flattened and ready to be serialized.
#[derive(Debug, Default)]
pub struct Module {
    pub glyphs: Vec<GlyphItem>,
    pub items: Vec<FlatSvgItem>,
}

impl Module {
    /// Get a glyph item by its stable ref.
    pub fn get_glyph(&self, id: &WeakAbsoulteRef) -> Option<&GlyphItem> {
        self.glyphs.get(id.0.id.0 as usize)
    }

    /// Get a svg item by its stable ref.
    pub fn get_item(&self, id: &AbsoulteRef) -> Option<&FlatSvgItem> {
        self.items.get(id.id.0 as usize)
    }
}

/// A Svg item that is specialized for representing [`typst::doc::Document`] or its subtypes.
#[derive(Debug, Clone)]
pub enum SvgItem {
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(TextItem),
    Transformed(TransformedItem),
    Group(GroupItem),
}

impl SvgItem {
    pub fn flatten(self) -> (AbsoulteRef, Module) {
        let mut builder = ModuleBuilder::default();

        let entry_id = builder.build(self);
        (entry_id, builder.finalize())
    }
}

/// Item representing an `<image/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ImageItem {
    pub image: Image,
    pub size: Size,
}

/// Item representing an `<a/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LinkItem {
    pub href: ImmutStr,
    pub size: Size,
}

/// Item representing an `<path/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PathItem {
    pub d: ImmutStr,
    pub styles: Vec<PathStyle>,
}

/// Attributes that is applicable to the [`PathItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PathStyle {
    Fill(ImmutStr),
    Stroke(ImmutStr),
    StrokeLineCap(ImmutStr),
    StrokeLineJoin(ImmutStr),
    StrokeMitterLimit(Scalar),
    StrokeDashOffset(Abs),
    StrokeDashArray(Arc<[Abs]>),
    StrokeWidth(Abs),
}

/// Item representing an `<g><text/><g/>` element.
#[derive(Debug, Clone)]
pub struct TextItem {
    pub content: Arc<TextItemContent>,
    pub shape: Arc<TextShape>,
}

/// The content metadata of a [`TextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextItemContent {
    pub content: ImmutStr,
    /// The glyphs in the text.
    /// (offset, advance, glyph): ([`Abs`], [`Abs`], [`GlyphItem`])
    pub glyphs: Vec<(Abs, Abs, GlyphItem)>,
}

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GlyphItem {
    // Failed,
    Raw(Font, GlyphId),
}

/// The shape metadata of a [`TextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextShape {
    pub dir: Dir,
    pub ascender: Abs,
    pub upem: Scalar,
    pub ppem: Scalar,
    pub fill: ImmutStr,
}

#[derive(Debug, Clone)]
pub enum TextStyle {}

/// Item representing an `<g/>` element applied with a [`TransformItem`].
#[derive(Debug, Clone)]
pub struct TransformedItem(pub TransformItem, pub Box<SvgItem>);

/// Absolute positioning items at their corresponding points.
#[derive(Debug, Clone)]
pub struct GroupItem(pub Vec<(Point, SvgItem)>);

/// Item representing all the transform that is applicable to a [`SvgItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TransformItem {
    Matrix(Arc<Transform>),
    Translate(Arc<Axes<Abs>>),
    Scale(Arc<(Ratio, Ratio)>),
    Rotate(Arc<Scalar>),
    Skew(Arc<(Ratio, Ratio)>),
    Clip(Arc<PathItem>),
}

/// Flatten svg item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FlatSvgItem {
    None,
    Glyph(GlyphItem),
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(FlatTextItem),
    Item(TransformedRef),
    Group(GroupRef),
}

/// Flatten text item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FlatTextItem {
    pub content: Arc<FlatTextItemContent>,
    pub shape: Arc<TextShape>,
}

/// The content metadata of a [`FlatTextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FlatTextItemContent {
    pub content: ImmutStr,
    pub glyphs: Arc<[(Abs, Abs, WeakAbsoulteRef)]>,
}

/// Flatten transform item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TransformedRef(pub TransformItem, pub RelativeRef);

/// Flatten group item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GroupRef(pub Arc<[(Point, RelativeRef)]>);

/// Global style namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StyleNs {
    Fill,
}

struct FingerprintSipHasher {
    data: Vec<u8>,
}

impl FingerprintHasher for FingerprintSipHasher {
    fn finish_fingerprint(&self) -> (u64, Vec<u8>) {
        let buffer = self.data.clone();
        let mut inner = FxHasher::default();
        buffer.hash(&mut inner);
        let hash = inner.finish();
        (hash, buffer)
    }
}

impl std::hash::Hasher for FingerprintSipHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn finish(&self) -> u64 {
        let buffer = Buffer::from(self.data.clone());
        let mut inner = SipHasher13::new();
        buffer.hash(&mut inner);
        inner.finish()
    }
}

#[derive(Default)]
pub struct GlobalFingerprintBuilder {
    unique_map: HashMap<Vec<u8>, Fingerprint>,
    disambiguators: HashMap<u64, u64>,
}

impl GlobalFingerprintBuilder {
    fn get_disambiguator(&mut self, fingerprint_hash: u64) -> u64 {
        let disambiguator = self.disambiguators.entry(fingerprint_hash).or_insert(0);
        *disambiguator += 1;
        *disambiguator
    }

    pub fn resolve<T: Hash + 'static>(&mut self, item: &T) -> Fingerprint {
        let mut s = FingerprintSipHasher { data: Vec::new() };
        item.type_id().hash(&mut s);
        item.hash(&mut s);
        let fingerprint_hash = s.finish_fingerprint();
        if let Some(fingerprint) = self.unique_map.get(&fingerprint_hash.1) {
            return *fingerprint;
        }

        let disambiguator = self.get_disambiguator(fingerprint_hash.0);
        let fingerprint = Fingerprint(fingerprint_hash.0, disambiguator);
        self.unique_map.insert(fingerprint_hash.1, fingerprint);
        fingerprint
    }
}

pub static GLOBAL_FINGERPRINT_BUILDER: Lazy<Arc<Mutex<GlobalFingerprintBuilder>>> =
    Lazy::new(|| Arc::new(Mutex::new(GlobalFingerprintBuilder::default())));

/// Intermediate representation of a incompleted svg item.
#[derive(Default)]
pub struct ModuleBuilder {
    pub glyph_uniquer: HashMap<GlyphItem, WeakAbsoulteRef>,
    pub disambiguators: HashMap<u128, u64>,
    pub items: Vec<FlatSvgItem>,
}

impl ModuleBuilder {
    pub fn finalize(self) -> Module {
        let mut glyphs = self.glyph_uniquer.into_iter().collect::<Vec<_>>();
        glyphs.sort_by(|(_, a), (_, b)| a.0.id.0.cmp(&b.0.id.0));
        Module {
            items: self.items,
            glyphs: glyphs.into_iter().map(|(a, _)| a).collect(),
        }
    }

    pub fn build_glyph(&mut self, glyph: GlyphItem) -> WeakAbsoulteRef {
        if let Some(id) = self.glyph_uniquer.get(&glyph) {
            return id.clone();
        }

        let id = DefId(self.glyph_uniquer.len() as u64);

        let fingerprint = GLOBAL_FINGERPRINT_BUILDER.lock().unwrap().resolve(&glyph);
        let rel_ref = WeakAbsoulteRef(AbsoulteRef { fingerprint, id });
        self.glyph_uniquer.insert(glyph, rel_ref.clone());
        rel_ref
    }

    pub fn build(&mut self, item: SvgItem) -> AbsoulteRef {
        let id = DefId(self.items.len() as u64);
        self.items.push(FlatSvgItem::None);

        let resolved_item = match item {
            SvgItem::Image(image) => FlatSvgItem::Image(image),
            SvgItem::Path(path) => FlatSvgItem::Path(path),
            SvgItem::Link(link) => FlatSvgItem::Link(link),
            SvgItem::Text(text) => {
                let glyphs = text
                    .content
                    .glyphs
                    .iter()
                    .cloned()
                    .map(|(offset, advance, glyph)| (offset, advance, self.build_glyph(glyph)))
                    .collect::<Arc<_>>();
                let shape = text.shape.clone();
                let content = text.content.content.clone();
                FlatSvgItem::Text(FlatTextItem {
                    content: Arc::new(FlatTextItemContent { content, glyphs }),
                    shape,
                })
            }
            SvgItem::Transformed(transformed) => {
                let item = &transformed.1;
                let item_id = self.build(*item.clone());
                let transform = transformed.0.clone();

                FlatSvgItem::Item(TransformedRef(transform, id.make_relative_ref(item_id)))
            }
            SvgItem::Group(group) => {
                let items = group
                    .0
                    .iter()
                    .map(|(point, item)| (*point, id.make_relative_ref(self.build(item.clone()))))
                    .collect::<Vec<_>>();
                FlatSvgItem::Group(GroupRef(items.into()))
            }
        };

        let fingerprint = GLOBAL_FINGERPRINT_BUILDER
            .lock()
            .unwrap()
            .resolve(&resolved_item);

        self.items[id.0 as usize] = resolved_item;
        AbsoulteRef { fingerprint, id }
    }
}
