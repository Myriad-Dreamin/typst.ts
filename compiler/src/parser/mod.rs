mod modifier_set;
mod semantic_tokens;
// mod token_encode;
mod typst_tokens;

use typst::{diag::FileResult, syntax::Source};

use typst_ts_core::TypstFileId;

pub use semantic_tokens::{
    get_semantic_tokens_full, get_semantic_tokens_legend, OffsetEncoding, SemanticToken,
    SemanticTokensLegend,
};

pub fn reparse(source_id: TypstFileId, prev: Option<Source>, next: String) -> FileResult<Source> {
    use dissimilar::Chunk;
    match prev {
        Some(mut source) => {
            let prev = source.text();
            if prev == next {
                Ok(source)
            } else {
                let prev = prev.to_owned();

                let diff = dissimilar::diff(&prev, &next);

                if diff.len() == 1 {
                    match diff[0] {
                        Chunk::Insert(_) => {
                            return Ok(Source::new(source_id, next));
                        }
                        Chunk::Delete(_) => {
                            return Ok(Source::new(source_id, "".to_owned()));
                        }
                        Chunk::Equal(_) => unreachable!(),
                    }
                }

                let mut rev_adavance = 0;
                let mut last_rep = false;
                let prev_len = prev.len();
                for op in diff.iter().rev().zip(diff.iter().rev().skip(1)) {
                    if last_rep {
                        last_rep = false;
                        continue;
                    }
                    match op {
                        (Chunk::Delete(t), Chunk::Insert(s))
                        | (Chunk::Insert(s), Chunk::Delete(t)) => {
                            rev_adavance += t.len();
                            source.edit(
                                prev_len - rev_adavance..prev_len - rev_adavance + t.len(),
                                s,
                            );
                            last_rep = true;
                        }
                        (Chunk::Delete(t), Chunk::Equal(e)) => {
                            rev_adavance += t.len();
                            source.edit(
                                prev_len - rev_adavance..prev_len - rev_adavance + t.len(),
                                "",
                            );
                            rev_adavance += e.len();
                            last_rep = true;
                        }
                        (Chunk::Insert(s), Chunk::Equal(e)) => {
                            source.edit(prev_len - rev_adavance..prev_len - rev_adavance, s);
                            last_rep = true;
                            rev_adavance += e.len();
                        }
                        (Chunk::Equal(t), _) => {
                            rev_adavance += t.len();
                        }
                        _ => unreachable!(),
                    }
                }

                if !last_rep {
                    match diff[0] {
                        Chunk::Insert(s) => {
                            source.edit(0..0, s);
                        }
                        Chunk::Delete(s) => {
                            source.edit(0..s.len(), "");
                        }
                        Chunk::Equal(_) => {}
                    }
                }

                Ok(source)
            }
        }
        None => Ok(Source::new(source_id, next)),
    }
}

#[cfg(test)]
mod tests {
    use typst::syntax::VirtualPath;
    use typst_ts_core::TypstFileId;

    #[track_caller]
    fn assert_same_ast(a: &typst::syntax::SyntaxNode, b: &typst::syntax::SyntaxNode) {
        assert_eq!(a.text(), b.text());
        assert_eq!(format!("{:#?}", a), format!("{:#?}", b));
    }

    #[test]
    fn test_reparse_add_prefix_suffix() {
        use super::reparse;
        let path = VirtualPath::new("main.typ");
        let source_id = TypstFileId::new(None, path);
        let empty = reparse(source_id, None, "".to_owned()).unwrap();
        let with_ba = reparse(source_id, None, "ba".to_owned()).unwrap();

        let edit_a = reparse(source_id, Some(empty.clone()), "a".to_owned()).unwrap();
        let edit_ba = reparse(source_id, Some(edit_a.clone()), "ba".to_owned()).unwrap();
        assert_same_ast(with_ba.root(), edit_ba.root());

        let edit_b = reparse(source_id, Some(empty.clone()), "b".to_owned()).unwrap();
        let edit_ba = reparse(source_id, Some(edit_b.clone()), "ba".to_owned()).unwrap();
        assert_same_ast(with_ba.root(), edit_ba.root());

        let with_aba = reparse(source_id, None, "aba".to_owned()).unwrap();

        let edit_aba = reparse(source_id, Some(edit_b), "aba".to_owned()).unwrap();
        assert_same_ast(with_aba.root(), edit_aba.root());

        let edit_aba = reparse(source_id, Some(edit_a), "aba".to_owned()).unwrap();
        assert_same_ast(with_aba.root(), edit_aba.root());

        let edit_aba = reparse(source_id, Some(empty), "aba".to_owned()).unwrap();
        assert_same_ast(with_aba.root(), edit_aba.root());
    }

    #[test]
    fn test_reparse_multiple_selection() {
        use super::reparse;
        let path = VirtualPath::new("main.typ");
        let source_id = TypstFileId::new(None, path);
        let empty = reparse(source_id, None, "".to_owned()).unwrap();
        let with_ba = reparse(
            source_id,
            None,
            "Long TeX TeX, TeX, It is a long text".to_owned(),
        )
        .unwrap();

        let edit_a = reparse(
            source_id,
            Some(empty.clone()),
            "Long Text Text, Text, It is long text".to_owned(),
        )
        .unwrap();
        let edit_ba = reparse(
            source_id,
            Some(edit_a.clone()),
            "Long TeX TeX, TeX, It is a long text".to_owned(),
        )
        .unwrap();
        assert_same_ast(with_ba.root(), edit_ba.root());

        let edit_a = reparse(
            source_id,
            Some(empty.clone()),
            "Long  , , It is long text".to_owned(),
        )
        .unwrap();
        let edit_ba = reparse(
            source_id,
            Some(edit_a.clone()),
            "Long TeX TeX, TeX, It is a long text".to_owned(),
        )
        .unwrap();
        assert_same_ast(with_ba.root(), edit_ba.root());

        let edit_a = reparse(
            source_id,
            Some(empty.clone()),
            "Long LaTeX LaTeX, LaTeX, It is long text".to_owned(),
        )
        .unwrap();
        let edit_ba = reparse(
            source_id,
            Some(edit_a.clone()),
            "Long TeX TeX, TeX, It is a long text".to_owned(),
        )
        .unwrap();
        assert_same_ast(with_ba.root(), edit_ba.root());
    }

    #[test]
    fn test_reparse_issue_typst_preview_vscode_issues_59() {
        use super::reparse;
        let path = VirtualPath::new("main.typ");
        let source_id = TypstFileId::new(None, path);
        let empty = reparse(source_id, None, "".to_owned()).unwrap();
        let with_a = reparse(source_id, None, "a".to_owned()).unwrap();
        let edit_a = reparse(source_id, Some(empty), "a".to_owned()).unwrap();
        assert_same_ast(with_a.root(), edit_a.root());
    }
}
