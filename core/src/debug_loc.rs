pub use reflexo::debug_loc::*;

pub use typst::layout::Position as TypstPosition;

/// Unevaluated source span.
/// The raw source span is unsafe to serialize and deserialize.
/// Because the real source location is only known during liveness of
/// the compiled document.
pub type SourceSpan = typst::syntax::Span;

/// Unevaluated source span with offset.
///
/// It adds an additional offset relative to the start of the span.
///
/// The offset is usually generated when the location is inside of some
/// text or string content.
#[derive(Debug, Clone, Copy)]
pub struct SourceSpanOffset {
    pub span: SourceSpan,
    pub offset: usize,
}

/// Lifts a [`SourceSpan`] to [`SourceSpanOffset`].
impl From<SourceSpan> for SourceSpanOffset {
    fn from(span: SourceSpan) -> Self {
        Self { span, offset: 0 }
    }
}

/// Converts a [`SourceSpan`] and an in-text offset to [`SourceSpanOffset`].
impl From<(SourceSpan, u16)> for SourceSpanOffset {
    fn from((span, offset): (SourceSpan, u16)) -> Self {
        Self {
            span,
            offset: offset as usize,
        }
    }
}
