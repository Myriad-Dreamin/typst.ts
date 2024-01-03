use std::path::Path;

use typst::model::Document;
use typst_ts_compiler::{
    service::{CompileDriver, CompileExporter, Compiler},
    ShadowApi, TypstSystemWorld,
};
use typst_ts_core::{
    config::CompileOpts,
    exporter_builtins::GroupExporter,
    vector::{
        incr::{IncrDocClient, IncrDocServer},
        ir::{Abs, Point, Rect},
        stream::BytesModuleStream,
    },
};
use typst_ts_svg_exporter::IncrSvgDocClient;

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<Document>,
) -> CompileExporter<CompileDriver> {
    let project_base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let font_path = project_base.join("assets/fonts");
    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: workspace_dir.to_owned(),
        no_system_fonts: true,
        font_paths: vec![font_path],
        ..CompileOpts::default()
    })
    .unwrap();

    let driver = CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
    };

    CompileExporter::new(driver).with_exporter(exporter)
}

pub fn test_compiler(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<Document>,
) {
    let mut driver = get_driver(workspace_dir, entry_file_path, exporter);
    let mut content = { std::fs::read_to_string(entry_file_path).expect("Could not read file") };

    let mut incr_server = IncrDocServer::default();
    let mut incr_client = IncrDocClient::default();
    let mut incr_svg_client = IncrSvgDocClient::default();

    let window = Rect {
        lo: Point::new(Abs::from(0.), Abs::from(0.)),
        hi: Point::new(Abs::from(1e33), Abs::from(1e33)),
    };

    let mut diff = vec![];

    // checkout the entry file
    let main_id = driver.main_id();

    let doc = driver
        .with_shadow_file_by_id(main_id, content.as_bytes().into(), |driver| {
            driver.compile(&mut Default::default())
        })
        .unwrap();
    let server_delta = incr_server.pack_delta(doc);
    let server_delta = BytesModuleStream::from_slice(&server_delta).checkout_owned();
    incr_client.merge_delta(server_delta);

    for i in 0..200 {
        println!("Iteration {}", i);

        content = content.replace("@netwok2020", "@netwok2020 x");

        let doc = driver
            .with_shadow_file_by_id(main_id, content.as_bytes().into(), |driver| {
                driver.compile(&mut Default::default())
            })
            .unwrap();

        let server_delta = incr_server.pack_delta(doc);
        let sd = server_delta.len();
        let server_delta = BytesModuleStream::from_slice(&server_delta).checkout_owned();
        incr_client.merge_delta(server_delta);
        incr_client.set_layout(incr_client.doc.layouts[0].unwrap_single());
        let cd = incr_svg_client.render_in_window(&mut incr_client, window);
        // std::fs::write(format!("{}.svg", i), cd.clone()).unwrap();
        diff.push((sd, cd.len()));

        comemo::evict(10);
    }

    println!("diff: {:?}", diff);
}

pub fn main() {
    let workspace_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let entry_file_path = workspace_dir.join("fuzzers/corpora/typst-templates/ieee/main.typ");

    let noop_exporter = GroupExporter::new(vec![]);
    test_compiler(&workspace_dir, &entry_file_path, noop_exporter);
}
