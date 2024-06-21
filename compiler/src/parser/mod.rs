mod modifier_set;
mod semantic_tokens;
// mod token_encode;
mod typst_tokens;

pub use semantic_tokens::{
    get_semantic_tokens_full, get_semantic_tokens_legend, OffsetEncoding, SemanticToken,
    SemanticTokensLegend,
};
