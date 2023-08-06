use std::path::Path;

use typst::{diag::FileResult, syntax::Source};

use typst_ts_core::TypstFileId;

pub fn reparse(
    _path: &Path,
    source_id: TypstFileId,
    prev: Option<Source>,
    next: String,
) -> FileResult<Source> {
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

                Ok(source)
            }
        }
        None => Ok(Source::new(source_id, next)),
    }
}

#[cfg(test)]
mod tests {
    use typst_ts_core::TypstFileId;

    #[test]
    fn test_reparse_issue_typst_preview_vscode_issues_59() {
        use super::reparse;
        let path = std::path::Path::new("/main.typ");
        let source_id = TypstFileId::new(None, path);
        let empty = reparse(path, source_id, None, "".to_owned()).unwrap();
        let with_a = reparse(path, source_id, None, "a".to_owned()).unwrap();
        let edit_a = reparse(path, source_id, Some(empty), "a".to_owned()).unwrap();
        assert_eq!(with_a.root(), edit_a.root());
    }
}
