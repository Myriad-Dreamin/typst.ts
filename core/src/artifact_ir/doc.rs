use std::num::NonZeroUsize;

use serde::Deserialize;
use serde::Serialize;
pub use typst::doc::Document as TypstDocument;
pub use typst::doc::Frame as TypstFrame;
pub use typst::doc::Glyph as TypstGlyph;
pub use typst::doc::Lang as TypstLang;
pub use typst::doc::Meta as TypstMeta;

use super::core::EcoString;
use super::core::FontRef;
use super::core::GlyphShapeRef;
use super::core::HasItemRefKind;
use super::core::ItemArray;
use super::core::ItemRef;
use super::core::ItemRefKind;
use super::core::Lang;
use super::core::PaintRef;
use super::core::SpanRef;
use super::geom::Em;
use super::geom::Shape;
use super::geom::Transform;
use super::geom::{Abs, Point, Size};
use super::image::Image;

#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemWithPos {
    pub item: ItemRef<FrameItem>,
    pub pos: Point,
}

impl HasItemRefKind for ItemWithPos {
    const ITEM_REF_KIND: ItemRefKind = ItemRefKind::ItemWithPos;
}

#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    /// The size of the frame.
    pub size: Size,
    /// The baseline of the frame measured from the top. If this is `None`, the
    /// frame's implicit baseline is at the bottom.
    pub baseline: Option<Abs>,
    /// The items composing this layout.
    pub items: ItemArray<ItemWithPos>,
}

impl HasItemRefKind for Frame {
    const ITEM_REF_KIND: ItemRefKind = ItemRefKind::Frame;
}

/// The building block frames are composed of.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum FrameItem {
    Group(GroupItem),
    Text(TextItem),
    Shape(Shape),
    Image(Image, Size),
    MetaLink(Destination, Size),
    None,
}

impl HasItemRefKind for FrameItem {
    const ITEM_REF_KIND: ItemRefKind = ItemRefKind::FrameItem;
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupItem {
    /// The group's frame.
    pub frame: Frame,
    /// A transformation to apply to the group.
    pub transform: Transform,
    /// Whether the frame should be a clipping boundary.
    pub clips: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlyphShape {
    /// The glyph's index in the font.
    pub id: u16,
    /// The glyph's range width (utf-8).
    pub range_width: u16,
    /// The advance width of the glyph.
    pub x_advance: Em,
    /// The horizontal offset of the glyph.
    pub x_offset: Em,
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlyphItem {
    /// shape reference
    pub shape: GlyphShapeRef,
    /// The source code location of the text.
    pub span: (SpanRef, u16),
    /// The range of the glyph in its item's text.
    pub range_start: u16,
}

impl HasItemRefKind for GlyphItem {
    const ITEM_REF_KIND: ItemRefKind = ItemRefKind::Glyph;
}

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextItem {
    /// The font the glyphs are contained in.
    pub font: FontRef,
    /// The font size.
    pub size: Abs,
    /// Glyph color.
    pub fill: PaintRef,
    /// The natural language of the text.
    pub lang: ItemRef<Lang>,
    /// The item's plain text.
    pub text: ItemRef<String>,
    /// The glyphs.
    pub glyphs: ItemArray<GlyphItem>,
}

/// A link destination.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "v")]
pub enum Destination {
    /// A link to a URL.
    Url(ItemRef<EcoString>),
    /// A link to a point on a page.
    Position(Position),
    /// An unresolved link to a location in the document.
    Location(ItemRef<String>),
}

/// A physical position in a document.
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    /// The page, starting at 1.
    pub page: NonZeroUsize,
    /// The exact coordinates on the page (from the top left, as usual).
    pub point: Point,
}
