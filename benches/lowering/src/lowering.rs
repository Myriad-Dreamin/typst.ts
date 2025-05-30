use std::sync::{Arc, Mutex};

use divan::Bencher;
use reflexo_typst::{
    EntryReader, ShadowApiExt, TypstPagedDocument, TypstSystemUniverse, MEMORY_MAIN_ENTRY,
};
use reflexo_typst2vec::pass::{IncrTypst2VecPass, Typst2VecPass};
use std::sync::LazyLock;
use typst::foundations::Bytes;
use typst_ts_cli::CompileOnceArgs;

type CompileDriver = LazyLock<Mutex<TypstSystemUniverse>>;

static TEST_COMPILER: CompileDriver = LazyLock::new(|| {
    Mutex::new(typst_ts_cli::compile::resolve_universe(CompileOnceArgs {
        workspace: "/".to_owned(),
        entry: "/main.typ".to_owned(),
        ..Default::default()
    }))
});

const TEST_FILE: &str = include_str!("../../../fuzzers/corpora/math/undergradmath.typ");

static TEST_DOC: LazyLock<Arc<TypstPagedDocument>> =
    LazyLock::new(|| compile(&TEST_COMPILER, TEST_FILE));

fn compile(driver: &CompileDriver, src: &str) -> Arc<TypstPagedDocument> {
    let mut driver = driver.lock().unwrap();
    let e = driver.main_id().unwrap_or_else(|| *MEMORY_MAIN_ENTRY);
    driver
        .with_shadow_file_by_id(e, Bytes::new(src.as_bytes().to_vec()), |this| {
            this.computation().compile().output
        })
        .unwrap()
}

fn main() {
    // initialize global variables
    let _unused = TEST_COMPILER.lock().unwrap();
    drop(_unused);
    let _doc = TEST_DOC.clone();

    // Run registered benchmarks.
    divan::main();
}

fn lower_impl(doc: &TypstPagedDocument) {
    let pass = Typst2VecPass::default();
    let _ = pass.paged(doc);
}

fn lower_incr_impl<'a>(docs: impl Iterator<Item = &'a Arc<TypstPagedDocument>>) {
    let mut pass = IncrTypst2VecPass::default();
    for doc in docs {
        pass.increment_lifetime();
        // lower_builder.gc(5 * 2);
        let _ = pass.paged(doc);
        // comemo::evict(30);
        pass.spans.reset();
    }
    // comemo::evict(0);
}

// Check lowering performance with cache
#[divan::bench]
fn lower_cached() {
    lower_impl(&TEST_DOC);
}

// Check lowering performance without cache
#[divan::bench]
fn lower_uncached() {
    lower_impl(&TEST_DOC);
}

// Check lowering performance during user edition
#[divan::bench]
fn lower_incr(bencher: Bencher) {
    let file_contents = (0..32)
        .map(|i| TEST_FILE.to_owned() + &("\nTest Incr").repeat(i))
        .collect::<Vec<_>>();
    let docs = file_contents
        .iter()
        .map(|s| compile(&TEST_COMPILER, s))
        .collect::<Vec<_>>();

    bencher.bench_local(|| {
        lower_incr_impl(docs.iter());
    });
}

/*
v0.4.1-rc2
typst_ts_bench_lowering  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ lower_cached          720.3 µs      │ 1.634 ms      │ 870 µs        │ 902.1 µs      │ 100     │ 100
├─ lower_incr            23.55 ms      │ 30.62 ms      │ 24.69 ms      │ 24.94 ms      │ 100     │ 100
╰─ lower_uncached        741.3 µs      │ 1.343 ms      │ 804.1 µs      │ 855.2 µs      │ 100     │ 100

v0.4.1-rc3 with text item cache
typst_ts_bench_lowering  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ lower_cached          248.2 µs      │ 1.158 ms      │ 262.4 µs      │ 286.3 µs      │ 100     │ 100
├─ lower_incr            8.488 ms      │ 13.19 ms      │ 9.048 ms      │ 9.191 ms      │ 100     │ 100
├─ lower_the_thesis      972.7 ms      │ 1.555 s       │ 1.315 s       │ 1.29 s        │ 100     │ 100
╰─ lower_uncached        1.055 ms      │ 1.837 ms      │ 1.12 ms       │ 1.191 ms      │ 100     │ 100

v0.5.0-rc3, there is no comemo cache set anymore, so lower_cached is same as lower_uncached
typst_ts_bench_lowering  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ lower_cached          1.192 ms      │ 3.315 ms      │ 1.352 ms      │ 1.448 ms      │ 100     │ 100
├─ lower_incr            12.01 ms      │ 20.55 ms      │ 13.94 ms      │ 14.43 ms      │ 100     │ 100
├─ lower_the_thesis      421.5 ms      │ 568.7 ms      │ 500 ms        │ 495.5 ms      │ 100     │ 100
╰─ lower_uncached        1.156 ms      │ 2.839 ms      │ 1.398 ms      │ 1.479 ms      │ 100     │ 100
 */

#[cfg(feature = "the-thesis")]
static THE_THESIS_COMPILER: CompileDriver = std::sync::LazyLock::new(|| {
    use std::path::Path;
    use typst_ts_cli::FontArgs;
    let the_thesis_path =
        env!("CARGO_MANIFEST_DIR").to_owned() + "../../../../../typst/masterproef";
    Mutex::new(typst_ts_cli::compile::resolve_universe(CompileOnceArgs {
        workspace: the_thesis_path.clone(),
        entry: the_thesis_path.clone() + "/masterproef/main.typ",
        font: FontArgs {
            paths: vec![Path::new(&(the_thesis_path + "/fonts")).to_owned()],
        },
        ..Default::default()
    }))
});

// Check lowering performance for the thesis
#[cfg(feature = "the-thesis")]
#[divan::bench]
fn lower_the_thesis(bencher: Bencher) {
    if !cfg!(feature = "the-thesis") {
        return;
    }

    let test_file = {
        let compiler = THE_THESIS_COMPILER.lock().unwrap();
        std::fs::read_to_string(compiler.entry_file()).unwrap()
    };

    let file_contents = (0..32)
        .map(|i| test_file.clone() + &("\nTest Incr").repeat(i))
        .collect::<Vec<_>>();
    let docs = file_contents
        .iter()
        .map(|s| compile(&THE_THESIS_COMPILER, s))
        .collect::<Vec<_>>();

    bencher.bench_local(|| {
        lower_incr_impl(docs.iter());
    });
}
