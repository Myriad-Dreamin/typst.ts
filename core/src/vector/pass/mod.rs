pub mod glyph2vec;
pub mod span2vec;
pub mod typst2vec;

pub use glyph2vec::*;
pub use span2vec::*;
pub use typst2vec::*;

use typst::syntax::Span;

const DETACHED: u64 = 1;
const SPAN_BITS: u64 = 48;

// todo: more safe way to transfer span id across process
/// Note: this function may be removed in the future.
pub fn span_id_to_u64(span_id: &Span) -> u64 {
    span_id
        .id()
        .map(|file_id| ((file_id.into_raw() as u64) << SPAN_BITS) | span_id.number())
        .unwrap_or(DETACHED)
}

/// Note: this function may be removed in the future.
pub fn span_id_from_u64(span_id: u64) -> Option<Span> {
    use typst::syntax::FileId;
    let file_id = if span_id == DETACHED {
        return Some(Span::detached());
    } else {
        let file_id = (span_id >> SPAN_BITS) as u16;
        FileId::from_raw(file_id)
    };

    Span::new(file_id, span_id & ((1u64 << SPAN_BITS) - 1))
}

#[cfg(test)]
mod tests {
    use typst::syntax::FileId;
    use typst::syntax::Span;

    use super::DETACHED;
    use super::SPAN_BITS;
    use super::{span_id_from_u64, span_id_to_u64};

    #[test]
    fn test_convert_span_id_u64() {
        let file_id = FileId::from_raw(1);
        let span_id = Span::new(file_id, 2).unwrap();

        // test span -> u64
        assert_eq!(span_id_to_u64(&Span::detached()), DETACHED);
        assert_eq!(span_id_to_u64(&span_id), (1u64 << SPAN_BITS) | 2);

        // test u64 -> span
        assert_eq!(span_id_from_u64(DETACHED), Some(Span::detached()));
        assert_eq!(span_id_from_u64(span_id_to_u64(&span_id)), Some(span_id));

        // test invalid u64
        assert_eq!(span_id_from_u64(0), None);
    }
}
