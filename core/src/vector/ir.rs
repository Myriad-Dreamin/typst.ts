use core::fmt;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::{
    hash::{typst_affinite_hash, Fingerprint},
    StaticHash128,
};
use base64::Engine;
use ttf_parser::GlyphId;
use typst::{
    font::Font,
    image::{ImageFormat, RasterFormat, VectorFormat},
};

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize as rDeser, Serialize as rSer};

pub type ImmutStr = Arc<str>;

pub use super::geom::*;

/// The local id of a svg item.
/// This id is only unique within the svg document.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct DefId(pub u64);

/// A stable absolute reference.
/// The fingerprint is used to identify the item and likely unique between
/// different svg documents. The (local) def id is only unique within the svg
/// document.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct AbsoluteRef {
    /// The fingerprint of the item.
    pub fingerprint: Fingerprint,
    /// The local def id of the item.
    pub id: DefId,
}

/// Reference a font item in a more friendly format to compress and store
/// information, similar to [`GlyphRef`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct FontRef {
    /// The hash of the font to avoid collision.
    pub hash: u32,
    /// The local id of the font.
    pub idx: u32,
}

/// Reference a glyph item in a more friendly format to compress and store
/// information. The glyphs are locally stored in the svg module.
/// With a glyph reference, we can get both the font metric and the glyph data.
/// The `font_hash` is to let it safe to be cached, please see
/// [`crate::vector::flat_ir::FlatFontItem`] for more details.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct GlyphRef {
    /// The hash of the font to avoid collision.
    pub font_hash: u32,
    /// The local id of the glyph.
    pub glyph_idx: u32,
}

impl fmt::Debug for AbsoluteRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "<AbsRef: {}{}>",
            self.fingerprint.as_svg_id(""),
            self.id.0
        )
    }
}

impl Hash for AbsoluteRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fingerprint.hash(state);
    }
}

impl AbsoluteRef {
    /// Create a xml id from the given prefix and the def id of this reference.
    /// Note that the def id may not be stable across compilation.
    /// Note that the entire html document shares namespace for ids.
    #[comemo::memoize]
    fn as_unstable_svg_id_inner(id: u64, prefix: &'static str) -> String {
        let id = {
            let id = id.to_le_bytes();
            // truncate zero
            let rev_zero = id.iter().rev().skip_while(|&&b| b == 0).count();
            let id = &id[..rev_zero];
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(id)
        };
        [prefix, &id].join("")
    }

    #[inline]
    pub fn as_svg_id(&self, prefix: &'static str) -> String {
        self.fingerprint.as_svg_id(prefix)
    }

    #[inline]
    pub fn as_unstable_svg_id(&self, prefix: &'static str) -> String {
        Self::as_unstable_svg_id_inner(self.id.0, prefix)
    }
}

pub type SpanId = u64;

/// A Svg item that is specialized for representing [`typst::doc::Document`] or
/// its subtypes.
#[derive(Debug, Clone)]
pub enum SvgItem {
    Image((ImageItem, SpanId)),
    Link(LinkItem),
    Path((PathItem, SpanId)),
    Text(TextItem),
    Transformed(TransformedItem),
    Group(GroupItem),
}

/// Data of an `<image/>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct Image {
    /// The encoded image data.
    pub data: Vec<u8>,
    /// The format of the encoded `buffer`.
    pub format: ImmutStr,
    /// The size of the image.
    pub size: Axes<u32>,
    /// A text describing the image.
    pub alt: Option<ImmutStr>,
    /// prehashed image content.
    pub hash: Fingerprint,
}

/// Collect image data from [`typst::image::Image`].
impl From<typst::image::Image> for Image {
    fn from(image: typst::image::Image) -> Self {
        let format = match image.format() {
            ImageFormat::Raster(e) => match e {
                RasterFormat::Jpg => "jpeg",
                RasterFormat::Png => "png",
                RasterFormat::Gif => "gif",
            },
            ImageFormat::Vector(e) => match e {
                VectorFormat::Svg => "svg+xml",
            },
        };

        // steal prehash from [`typst::image::Image`]
        let hash = typst_affinite_hash(&image);

        Image {
            data: image.data().to_vec(),
            format: format.into(),
            size: image.size().into(),
            alt: image.alt().map(|s| s.into()),
            hash: Fingerprint::from_u128(hash),
        }
    }
}

impl Image {
    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.size.x
    }
    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.size.y
    }
}

/// Prehashed image data.
impl Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl StaticHash128 for Image {
    /// Returns the hash of the image data.
    fn get_hash(&self) -> u128 {
        self.hash.to_u128()
    }
}

/// Item representing an `<image/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct ImageItem {
    /// The source image data.
    pub image: Arc<Image>,
    /// The target size of the image.
    pub size: Size,
}

/// Item representing an `<a/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct LinkItem {
    /// The target of the link item.
    pub href: ImmutStr,
    /// The box size of the link item.
    pub size: Size,
}

/// Item representing an `<path/>` element.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct PathItem {
    /// The path instruction.
    pub d: ImmutStr,
    /// The path style.
    /// See [`PathStyle`] for more information.
    pub styles: Vec<PathStyle>,
}

/// Attributes that is applicable to the [`PathItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum PathStyle {
    /// `fill` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/fill>
    Fill(ImmutStr),

    /// `stroke` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke>
    Stroke(ImmutStr),

    /// `stroke-linecap` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke-linecap>
    StrokeLineCap(ImmutStr),

    /// `stroke-linejoin` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke-linejoin>
    StrokeLineJoin(ImmutStr),

    /// `stroke-miterlimit` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke-miterlimit>
    StrokeMitterLimit(Scalar),

    /// `stroke-dashoffset` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke-dashoffset>
    StrokeDashOffset(Abs),

    /// `stroke-dasharray` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke-dasharray>
    StrokeDashArray(Arc<[Abs]>),

    /// `stroke-width` attribute.
    /// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/stroke-width>
    StrokeWidth(Abs),
}

/// Item representing an `<g><text/><g/>` element.
#[derive(Debug, Clone)]
pub struct TextItem {
    /// The content of the text item.
    pub content: Arc<TextItemContent>,
    /// The shape of the text item.
    /// See [`TextShape`] for more information.
    pub shape: Arc<TextShape>,
}

/// The content metadata of a [`TextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextItemContent {
    /// The plain utf-8 content of the text item.
    /// Note: witout XML escaping.
    pub content: ImmutStr,
    /// The glyphs in the text.
    /// (offset, advance, glyph): ([`Abs`], [`Abs`], [`GlyphItem`])
    pub glyphs: Vec<(Abs, Abs, GlyphItem)>,
    /// Source span for this text item.
    pub span_id: u64,
}

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct ImageGlyphItem {
    pub ts: Transform,
    pub image: ImageItem,
}

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct OutlineGlyphItem {
    pub ts: Option<Transform>,
    pub d: ImmutStr,
}

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GlyphItem {
    /// Raw glyph representation.
    /// The raw glyphs is generated in lowering stage.
    Raw(Font, GlyphId),

    /// Glyphs in SVG or Bitmap format.
    Image(Arc<ImageGlyphItem>),

    /// Glyphs in path instructions, known as the "d" attribute of a
    /// `<path/>` element.
    Outline(Arc<OutlineGlyphItem>),
}

/// The shape metadata of a [`TextItem`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct TextShape {
    /// The direction of the text item.
    pub dir: ImmutStr,
    /// The ascent of the font used by the text item.
    pub ascender: Abs,
    /// The units per em of the font used by the text item.
    pub upem: Scalar,
    /// The pixels per em of the font used by the text item.
    pub ppem: Scalar,
    /// Fill font text with css color.
    pub fill: ImmutStr,
}

/// Item representing an `<g/>` element applied with a [`TransformItem`].
#[derive(Debug, Clone)]
pub struct TransformedItem(pub TransformItem, pub Box<SvgItem>);

/// Absolute positioning items at their corresponding points.
#[derive(Debug, Clone, Default)]
pub struct GroupItem(pub Vec<(Point, SvgItem)>);

/// Item representing all the transform that is applicable to a [`SvgItem`].
/// See <https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/transform>
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub enum TransformItem {
    /// `matrix` transform.
    Matrix(Arc<Transform>),
    /// `translate` transform.
    Translate(Arc<Axes<Abs>>),
    /// `scale` transform.
    Scale(Arc<(Ratio, Ratio)>),
    /// `rotate` transform.
    Rotate(Arc<Scalar>),
    /// `skewX skewY` transform.
    Skew(Arc<(Ratio, Ratio)>),

    /// clip path.
    Clip(Arc<PathItem>),
}

/// Global style namespace.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StyleNs {
    /// style that contains a single css rule: `fill: #color`.
    Fill,
}

/// A finished pack that stores all the glyph items.
pub type GlyphPack = Vec<(DefId, GlyphItem)>;

#[derive(Default, Clone)]
pub struct GlyphPackBuilder {
    /// Intermediate representation of an incompleted font pack.
    font_mapping: HashMap<Font, FontRef>,

    /// Intermediate representation of an incompleted glyph pack.
    glyph_defs: HashMap<GlyphItem, (GlyphRef, FontRef)>,
}

impl GlyphPackBuilder {
    pub fn finalize(&self) -> GlyphPack {
        let mut glyphs = self.glyph_defs.clone().into_iter().collect::<Vec<_>>();
        glyphs.sort_by(|(_, a), (_, b)| a.0.glyph_idx.cmp(&b.0.glyph_idx));
        glyphs
            .into_iter()
            .map(|(a, b)| (DefId(b.1.idx as u64), a))
            .collect()
    }
}

pub trait BuildGlyph {
    fn build_glyph(&mut self, glyph: &GlyphItem) -> GlyphRef;
}

impl BuildGlyph for GlyphPackBuilder {
    fn build_glyph(&mut self, glyph: &GlyphItem) -> GlyphRef {
        if let Some(id) = self.glyph_defs.get(glyph) {
            return id.0.clone();
        }

        let id = DefId(self.glyph_defs.len() as u64);

        // let fingerprint = self.fingerprint_builder.resolve(glyph);
        // let abs_ref = AbsoluteRef { fingerprint, id };
        // self.glyph_defs.insert(glyph.clone(), abs_ref.clone());
        // abs_ref
        todo!() // glyph things
    }
}
