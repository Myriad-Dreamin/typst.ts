use reflexo_typst::{Bytes, ShadowApi, TypstPagedDocument, TypstWorld};
use tinymist_project::{CompileFontArgs, CompileOnceArgs};

/// Runs snapshot tests.
#[macro_export]
macro_rules! snapshot_testing {
    ($name:expr, $f:expr) => {
        let name = $name;
        let name = if name.is_empty() { "playground" } else { name };
        let mut settings = $crate::Settings::new();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path(format!("fixtures/{name}/snaps"));
        settings.bind(|| {
            let glob_path = format!("fixtures/{name}/*.typ");
            $crate::glob!(&glob_path, |path| {
                let contents = std::fs::read_to_string(path).unwrap();
                #[cfg(windows)]
                let contents = contents.replace("\r\n", "\n");

                $crate::run_with_sources(&contents, $f);
            });
        });
    };
}

#[test]
fn test_lin() {
    // let verse =
    // typst_ts_cli::compile::resolve_universe(compile_args.compile.clone());

    let verse = CompileOnceArgs {
        input: Some("main.typ".to_owned()),
        font: CompileFontArgs {
            ignore_system_fonts: true,
            ..Default::default()
        },
        ..CompileOnceArgs::default()
    }
    .resolve_system()
    .expect("resolve system");

    let mut world = verse.snapshot();

    let main = world.main();
    world
        .map_shadow_by_id(main, Bytes::from_string("Hello World!"))
        .expect("map shadow");

    println!("hello world");

    snapshot_testing!("lin", |verse, _path| {
        let world = verse.snapshot();

        let doc = typst::compile::<TypstPagedDocument>(&world)
            .output
            .expect("compile document");

        // let mut buf = Vec::new();
        // doc.write_to(&mut buf).expect("write document");
        // let doc = String::from_utf8(buf).expect("utf8 decode");

        // // println!("{doc}");
        // assert_eq!(doc, "Hello World!");
    });
}
