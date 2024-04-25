use once_cell::sync::Lazy;
use typst::syntax::{Source, VirtualPath};
use typst_ts_compiler::parser::reparse as reparse_selected;
use typst_ts_core::diag::FileResult;
use typst_ts_core::TypstFileId;

const TEST_FILE: &str = include_str!("../../../fuzzers/corpora/math/undergradmath.typ");
static CHORE_CHANGE: Lazy<String> =
    once_cell::sync::Lazy::new(|| TEST_FILE.replace("density", "DensitY"));
static LARGE_CHANGE: Lazy<String> =
    once_cell::sync::Lazy::new(|| TEST_FILE.replace("Typst", "typsT"));

static TEST_DOC: Lazy<Source> = once_cell::sync::Lazy::new(|| {
    Source::new(
        TypstFileId::new_fake(VirtualPath::new("/main.typ")),
        TEST_FILE.to_owned(),
    )
});

fn main() {
    // initialize global variables
    let _doc = TEST_DOC.clone();
    assert!(TEST_FILE != LARGE_CHANGE.clone());
    assert!(TEST_FILE != CHORE_CHANGE.clone());
    assert!(LARGE_CHANGE.clone() != CHORE_CHANGE.clone());

    // Run registered benchmarks.
    divan::main();
}

enum ReparseApproach {
    Uncached,
    Naive,
    Dissimilar,
    Selected,
}

fn parse(doc: &Source, next: &str, approach: ReparseApproach) -> Source {
    match approach {
        ReparseApproach::Uncached => Source::new(doc.id(), next.to_owned()),
        ReparseApproach::Naive => {
            let mut res = doc.clone();
            res.replace(next);
            res
        }
        ReparseApproach::Dissimilar => {
            reparse(doc.id(), Some(doc.clone()), next.to_owned()).unwrap()
        }
        ReparseApproach::Selected => {
            reparse_selected(doc.id(), Some(doc.clone()), next.to_owned()).unwrap()
        }
    }
}

#[divan::bench]
fn parse_chore_change_uncached() {
    parse(&TEST_DOC, &CHORE_CHANGE, ReparseApproach::Uncached);
}

#[divan::bench]
fn parse_chore_change_naive() {
    parse(&TEST_DOC, &CHORE_CHANGE, ReparseApproach::Naive);
}

#[divan::bench]
fn parse_chore_change_dissimilar() {
    parse(&TEST_DOC, &CHORE_CHANGE, ReparseApproach::Dissimilar);
}

#[divan::bench]
fn parse_chore_change_selected() {
    parse(&TEST_DOC, &CHORE_CHANGE, ReparseApproach::Selected);
}

#[divan::bench]
fn parse_large_change_uncached() {
    parse(&TEST_DOC, &LARGE_CHANGE, ReparseApproach::Uncached);
}

#[divan::bench]
fn parse_large_change_naive() {
    parse(&TEST_DOC, &LARGE_CHANGE, ReparseApproach::Naive);
}

#[divan::bench]
fn parse_large_change_dissimilar() {
    parse(&TEST_DOC, &LARGE_CHANGE, ReparseApproach::Dissimilar);
}

#[divan::bench]
fn parse_large_change_selected() {
    parse(&TEST_DOC, &LARGE_CHANGE, ReparseApproach::Selected);
}

/*
v0.5.0-rc3
typst_ts_bench_reparsing          fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ parse_chore_change_dissimilar  319.3 µs      │ 903.5 µs      │ 350.9 µs      │ 391 µs        │ 100     │ 100
├─ parse_chore_change_naive       122.3 µs      │ 405.4 µs      │ 154.7 µs      │ 166.2 µs      │ 100     │ 100
├─ parse_chore_change_uncached    794.9 µs      │ 2.258 ms      │ 920.1 µs      │ 1.024 ms      │ 100     │ 100
├─ parse_large_change_dissimilar  1.619 ms      │ 3.027 ms      │ 1.805 ms      │ 1.89 ms       │ 100     │ 100
├─ parse_large_change_naive       694.1 µs      │ 1.853 ms      │ 916.5 µs      │ 1.027 ms      │ 100     │ 100
╰─ parse_large_change_uncached    628.2 µs      │ 1.55 ms       │ 803.9 µs      │ 821.4 µs      │ 100     │ 100
 */

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
