use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use base64::Engine;
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};
use siphasher::sip128::{Hasher128, SipHasher13};
use ttf_parser::GlyphId;
use typst::font::Font;
use typst::image::{ImageFormat, RasterFormat, VectorFormat};
use typst_ts_core::typst_affinite_hash;

pub type ImmutStr = Arc<str>;

pub use super::geom::*;
/// See <https://github.com/rust-lang/rust/blob/master/compiler/rustc_hir/src/stable_hash_impls.rs#L22>
/// The fingerprint conflicts should be very rare and should be handled by the compiler.
///
/// > That being said, given a high quality hash function, the collision
/// > probabilities in question are very small. For example, for a big crate like
/// > `rustc_middle` (with ~50000 `LocalDefId`s as of the time of writing) there
/// > is a probability of roughly 1 in 14,750,000,000 of a crate-internal
/// > collision occurring. For a big crate graph with 1000 crates in it, there is
/// > a probability of 1 in 36,890,000,000,000 of a `StableCrateId` collision.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct Fingerprint(u64, u64);

pub trait FingerprintHasher: std::hash::Hasher {
    fn finish_fingerprint(&self) -> (Fingerprint, Vec<u8>);
}

struct FingerprintSipHasher {
    data: Vec<u8>,
}

impl std::hash::Hasher for FingerprintSipHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn finish(&self) -> u64 {
        let buffer = self.data.clone();
        let mut inner = SipHasher13::new();
        buffer.hash(&mut inner);
        inner.finish()
    }
}

impl FingerprintHasher for FingerprintSipHasher {
    fn finish_fingerprint(&self) -> (Fingerprint, Vec<u8>) {
        let buffer = self.data.clone();
        let mut inner = SipHasher13::new();
        buffer.hash(&mut inner);
        let hash = inner.finish128();
        (Fingerprint(hash.h1, hash.h2), buffer)
    }
}

#[derive(Default)]
pub struct FingerprintBuilder {
    conflict_checker: HashMap<Fingerprint, Vec<u8>>,
}

impl FingerprintBuilder {
    pub fn resolve<T: Hash + 'static>(&mut self, item: &T) -> Fingerprint {
        let mut s = FingerprintSipHasher { data: Vec::new() };
        item.type_id().hash(&mut s);
        item.hash(&mut s);
        let (fingerprint, featured_data) = s.finish_fingerprint();
        if let Some(prev_featured_data) = self.conflict_checker.get(&fingerprint) {
            if prev_featured_data != &featured_data {
                // todo: soft error
                panic!("Fingerprint conflict detected!");
            }

            return fingerprint;
        }

        self.conflict_checker.insert(fingerprint, featured_data);
        fingerprint
    }
}

/// The local id of a svg item.
/// This id is only unique within the svg document.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
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
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct RelativeDefId(pub i64);

/// A stable absolute reference.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct AbsoulteRef {
    pub fingerprint: Fingerprint,
    pub id: DefId,
}

impl Hash for AbsoulteRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fingerprint.hash(state);
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

/// A stable relative reference.
/// These objects can only be constructed relative from a [`AbsoulteRef`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct RelativeRef {
    pub fingerprint: Fingerprint,
    pub id: RelativeDefId,
}

impl Hash for RelativeRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fingerprint.hash(state);
    }
}

impl RelativeRef {
    pub(crate) fn as_svg_id(&self, prefix: &'static str) -> String {
        AbsoulteRef::as_svg_id_inner(self.fingerprint, prefix)
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct ItemPack(pub Vec<FlatSvgItem>);

/// A finished module that stores all the svg items.
/// The svg items shares the underlying data.
/// The svg items are flattened and ready to be serialized.
#[derive(Debug, Default)]
pub struct Module {
    pub glyphs: Vec<GlyphItem>,
    pub item_pack: ItemPack,
}

impl Module {
    /// Get a glyph item by its stable ref.
    pub fn get_glyph(&self, id: &AbsoulteRef) -> Option<&GlyphItem> {
        self.glyphs.get(id.id.0 as usize)
    }

    /// Get a svg item by its stable ref.
    pub fn get_item(&self, id: &AbsoulteRef) -> Option<&FlatSvgItem> {
        self.item_pack.0.get(id.id.0 as usize)
    }
}

/// Module with page references of a [`typst::doc::Document`].
pub struct SvgDocument {
    pub module: Module,
    /// References to the page frames.
    /// Use [`Module::get_item`] to get the actual item.
    pub pages: Vec<(AbsoulteRef, Size)>,
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
        (entry_id, builder.finalize().0)
    }
}

/// Data of an `<image/>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct Image {
    pub data: Vec<u8>,
    /// The format of the encoded `buffer`.
    pub format: ImmutStr,
    /// The size of the image.
    pub size: Axes<u32>,
    /// A text describing the image.
    pub alt: Option<ImmutStr>,
    /// prehashed image content.
    pub hash: u128,
}

impl From<typst::image::Image> for Image {
    fn from(image: typst::image::Image) -> Self {
        Image {
            data: image.data().to_vec(),
            format: match image.format() {
                ImageFormat::Raster(e) => match e {
                    RasterFormat::Jpg => "jpeg",
                    RasterFormat::Png => "png",
                    RasterFormat::Gif => "gif",
                },
                ImageFormat::Vector(e) => match e {
                    VectorFormat::Svg => "svg+xml",
                },
            }
            .into(),
            size: image.size().into(),
            alt: image.alt().map(|s| s.into()),
            hash: typst_affinite_hash(&image),
        }
    }
}

impl Image {
    pub fn width(&self) -> u32 {
        self.size.x
    }
    pub fn height(&self) -> u32 {
        self.size.y
    }
}

impl Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

/// Item representing an `<image/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct ImageItem {
    pub image: Arc<Image>,
    pub size: Size,
}

/// Item representing an `<a/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct LinkItem {
    pub href: ImmutStr,
    pub size: Size,
}

/// Item representing an `<path/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct PathItem {
    pub d: ImmutStr,
    pub styles: Vec<PathStyle>,
}

/// Attributes that is applicable to the [`PathItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
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
pub struct ImageGlyphItem {
    pub ts: Transform,
    pub image: ImageItem,
}

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutlineGlyphItem {
    pub ts: Option<Transform>,
    pub d: ImmutStr,
}

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GlyphItem {
    // Failed,
    Raw(Font, GlyphId),
    Image(Arc<ImageGlyphItem>),
    Outline(Arc<OutlineGlyphItem>),
}

/// The shape metadata of a [`TextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct TextShape {
    // todo: save direction
    // pub dir: Dir,
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
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
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
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub enum FlatSvgItem {
    None,
    Image(ImageItem),
    Link(LinkItem),
    Path(PathItem),
    Text(FlatTextItem),
    Item(TransformedRef),
    Group(GroupRef),
}

/// Flatten text item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct FlatTextItem {
    pub content: Arc<FlatTextItemContent>,
    pub shape: Arc<TextShape>,
}

/// The content metadata of a [`FlatTextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct FlatTextItemContent {
    pub content: ImmutStr,
    pub glyphs: Arc<[(Abs, Abs, AbsoulteRef)]>,
}

/// Flatten transform item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct TransformedRef(pub TransformItem, pub RelativeRef);

/// Flatten group item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
pub struct GroupRef(pub Arc<[(Point, RelativeRef)]>);

/// Global style namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StyleNs {
    Fill,
}

pub type GlyphMapping = HashMap<GlyphItem, AbsoulteRef>;

/// Intermediate representation of a incompleted svg item.
#[derive(Default)]
pub struct ModuleBuilder {
    pub glyphs: GlyphMapping,
    pub items: Vec<FlatSvgItem>,

    fingerprint_builder: FingerprintBuilder,
}

impl ModuleBuilder {
    pub fn finalize(self) -> (Module, GlyphMapping) {
        let mut glyphs = self.glyphs.clone().into_iter().collect::<Vec<_>>();
        glyphs.sort_by(|(_, a), (_, b)| a.id.0.cmp(&b.id.0));
        (
            Module {
                glyphs: glyphs.into_iter().map(|(a, _)| a).collect(),
                item_pack: ItemPack(self.items),
            },
            self.glyphs,
        )
    }

    pub fn build_glyph(&mut self, glyph: GlyphItem) -> AbsoulteRef {
        if let Some(id) = self.glyphs.get(&glyph) {
            return id.clone();
        }

        let id = DefId(self.glyphs.len() as u64);

        let fingerprint = self.fingerprint_builder.resolve(&glyph);
        let abs_ref = AbsoulteRef { fingerprint, id };
        self.glyphs.insert(glyph, abs_ref.clone());
        abs_ref
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

        let fingerprint = self.fingerprint_builder.resolve(&resolved_item);
        self.items[id.0 as usize] = resolved_item;
        AbsoulteRef { fingerprint, id }
    }
}
