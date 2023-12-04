use divan::Bencher;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use typst_ts_cli::{CompileOnceArgs, FontArgs};

use once_cell::sync::Lazy;
use typst::model::Document;
use typst_ts_compiler::{
    service::{CompileDriverImpl, Compiler},
    ShadowApi, TypstSystemWorld,
};
use typst_ts_core::vector::LowerBuilder;

type CompileDriver = Lazy<Mutex<CompileDriverImpl<TypstSystemWorld>>>;

static TEST_COMPILER: CompileDriver = once_cell::sync::Lazy::new(|| {
    Mutex::new(typst_ts_cli::compile::create_driver(CompileOnceArgs {
        workspace: "/".to_owned(),
        entry: "/main.typ".to_owned(),
        ..Default::default()
    }))
});

const TEST_FILE: &str = include_str!("../../../fuzzers/corpora/math/undergradmath.typ");

static TEST_DOC: Lazy<Arc<Document>> =
    once_cell::sync::Lazy::new(|| compile(&TEST_COMPILER, TEST_FILE));

fn compile(driver: &CompileDriver, src: &str) -> Arc<Document> {
    let mut driver = driver.lock().unwrap();
    let e = driver.entry_file.clone();
    driver
        .with_shadow_file(&e, src.as_bytes().into(), |this| {
            this.pure_compile(&mut Default::default())
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

// Check lowering performance with cache
#[divan::bench]
fn lower_cached() {
    let mut lower_builder = LowerBuilder::new(&TEST_DOC);
    for f in TEST_DOC.pages.iter() {
        let _ = lower_builder.lower(f);
    }
}

// Check lowering performance without cache
#[divan::bench]
fn lower_uncached() {
    let mut lower_builder = LowerBuilder::new(&TEST_DOC);
    for f in TEST_DOC.pages.iter() {
        let _ = lower_builder.lower(f);
    }
    comemo::evict(0);
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

    comemo::evict(0);

    bencher.bench_local(|| {
        for doc in docs.iter() {
            let mut lower_builder = LowerBuilder::new(doc);
            for f in doc.pages.iter() {
                let _ = lower_builder.lower(f);
            }
            comemo::evict(30);
        }
        comemo::evict(0);
    });
}

// v0.4.1-rc3
// typst_ts_bench_lowering  fastest       │ slowest       │ median        │ mean          │ samples │ iters
// ├─ lower_cached          720.3 µs      │ 1.634 ms      │ 870 µs        │ 902.1 µs      │ 100     │ 100
// ├─ lower_incr            23.55 ms      │ 30.62 ms      │ 24.69 ms      │ 24.94 ms      │ 100     │ 100
// ╰─ lower_uncached        741.3 µs      │ 1.343 ms      │ 804.1 µs      │ 855.2 µs      │ 100     │ 100

// v0.4.1-rc3 with text item cache
// typst_ts_bench_lowering  fastest       │ slowest       │ median        │ mean          │ samples │ iters
// ├─ lower_cached          248.2 µs      │ 1.158 ms      │ 262.4 µs      │ 286.3 µs      │ 100     │ 100
// ├─ lower_incr            8.488 ms      │ 13.19 ms      │ 9.048 ms      │ 9.191 ms      │ 100     │ 100
// ╰─ lower_uncached        1.055 ms      │ 1.837 ms      │ 1.12 ms       │ 1.191 ms      │ 100     │ 100

static THE_THESIS_COMPILER: CompileDriver = once_cell::sync::Lazy::new(|| {
    let the_thesis_path =
        env!("CARGO_MANIFEST_DIR").to_owned() + "../../../../../typst/masterproef";
    Mutex::new(typst_ts_cli::compile::create_driver(CompileOnceArgs {
        workspace: the_thesis_path.clone(),
        entry: the_thesis_path.clone() + "/masterproef/main.typ",
        font: FontArgs {
            paths: vec![Path::new(&(the_thesis_path + "/fonts")).to_owned()],
        },
        ..Default::default()
    }))
});

// Check lowering performance for the thesis
#[divan::bench]
fn lower_the_thesis(bencher: Bencher) {
    let test_file = {
        let compiler = THE_THESIS_COMPILER.lock().unwrap();
        let e = compiler.entry_file.clone();
        std::fs::read_to_string(Path::new(&e)).unwrap()
    };

    let file_contents = (0..32)
        .map(|i| test_file.clone() + &("\nTest Incr").repeat(i))
        .collect::<Vec<_>>();
    let docs = file_contents
        .iter()
        .map(|s| compile(&THE_THESIS_COMPILER, s))
        .collect::<Vec<_>>();

    comemo::evict(0);

    bencher.bench_local(|| {
        for doc in docs.iter() {
            let mut lower_builder = LowerBuilder::new(doc);
            for f in doc.pages.iter() {
                let _ = lower_builder.lower(f);
            }
            comemo::evict(30);
        }
        comemo::evict(0);
    });
}
