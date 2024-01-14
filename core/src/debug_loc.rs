use serde::{Deserialize, Serialize};
use typst::layout::Position as TypstPosition;

/// A serializable physical position in a document.
///
/// Note that it uses [`f32`] instead of [`f64`] as same as
/// [`TypstPosition`] for the coordinates to improve both performance
/// of serialization and calculation. It does sacrifice the floating
/// precision, but it is enough in our use cases.
///
/// Also see [`TypstPosition`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DocumentPosition {
    /// The page, starting at 1.
    pub page_no: usize,
    /// The exact x-coordinate on the page (from the left, as usual).
    pub x: f32,
    /// The exact y-coordinate on the page (from the top, as usual).
    pub y: f32,
}

impl From<TypstPosition> for DocumentPosition {
    fn from(position: TypstPosition) -> Self {
        Self {
            page_no: position.page.into(),
            x: position.point.x.to_pt() as f32,
            y: position.point.y.to_pt() as f32,
        }
    }
}

/// Unevaluated source span.
/// The raw source span is unsafe to serialize and deserialize.
/// Because the real source location is only known during liveness of
/// the compiled document.
pub type SourceSpan = typst::syntax::Span;

/// Raw representation of a source span.
pub type RawSourceSpan = u64;

/// A resolved source (text) location.
///
/// See [`CharPosition`] for the definition of the position inside a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLocation {
    pub filepath: String,
}

/// A char position represented in form of line and column.
/// The position is encoded in Utf-8 or Utf-16, and the encoding is
/// determined by usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPosition {
    /// The line number, starting at 0.
    pub line: usize,
    /// The column number, starting at 0.
    pub column: usize,
}

impl From<Option<(usize, usize)>> for CharPosition {
    fn from(loc: Option<(usize, usize)>) -> Self {
        let (start, end) = loc.unwrap_or_default();
        CharPosition {
            line: start,
            column: end,
        }
    }
}

/// A resolved source (text) location.
///
/// See [`CharPosition`] for the definition of the position inside a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub filepath: String,
    pub pos: CharPosition,
}

impl SourceLocation {
    pub fn from_flat(
        flat: FlatSourceLocation,
        i: &impl std::ops::Index<usize, Output = FileLocation>,
    ) -> Self {
        Self {
            filepath: i[flat.filepath as usize].filepath.clone(),
            pos: flat.pos,
        }
    }
}

/// A flat resolved source (text) location.
///
/// See [`CharPosition`] for the definition of the position inside a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatSourceLocation {
    pub filepath: u32,
    pub pos: CharPosition,
}

// /// A resolved file range.
// ///
// /// See [`CharPosition`] for the definition of the position inside a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharRange {
    pub start: CharPosition,
    pub end: CharPosition,
}

// /// A resolved source (text) range.
// ///
// /// See [`CharPosition`] for the definition of the position inside a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRange {
    pub path: String,
    pub range: CharRange,
}
