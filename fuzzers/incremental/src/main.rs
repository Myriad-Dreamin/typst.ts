use std::path::Path;

use reflexo_typst::config::{entry::EntryOpts, CompileOpts};
use reflexo_typst::exporter_builtins::GroupExporter;
use reflexo_typst::vector::{
    ir::{Abs, Point, Rect},
    stream::BytesModuleStream,
};
use reflexo_typst::TypstDocument;
use reflexo_typst::{
    CompileDriver, CompileExporter, PureCompiler, ShadowApiExt, TypstSystemUniverse,
    TypstSystemWorld,
};
use reflexo_typst2vec::incr::{IncrDocClient, IncrDocServer};
use reflexo_vec2svg::IncrSvgDocClient;
use typst_ts_incremental_fuzzer::mutate;

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<TypstDocument>,
) -> CompileDriver<CompileExporter<PureCompiler<TypstSystemWorld>>> {
    let project_base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let font_path = project_base.join("assets/fonts");
    let world = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.into()),
        no_system_fonts: true,
        font_paths: vec![font_path],
        ..CompileOpts::default()
    })
    .unwrap();

    let world = world.with_entry_file(entry_file_path.to_owned());
    CompileDriver::new(CompileExporter::default().with_exporter(exporter), world)
}

pub fn test_compiler(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<TypstDocument>,
) {
    let mut driver = get_driver(workspace_dir, entry_file_path, exporter);
    let mut content = { std::fs::read_to_string(entry_file_path).expect("Could not read file") };

    #[cfg(feature = "generate")]
    let mut incr_server = IncrDocServer::default();
    #[cfg(feature = "generate")]
    let mut incr_client = IncrDocClient::default();
    #[cfg(feature = "generate")]
    let mut incr_svg_client = IncrSvgDocClient::default();

    let window = Rect {
        lo: Point::new(Abs::from(0.), Abs::from(0.)),
        hi: Point::new(Abs::from(1e33), Abs::from(1e33)),
    };

    #[cfg(feature = "generate")]
    std::fs::write("mutate_sequence.log", "").unwrap();

    for i in 0..200 {
        println!("Iteration {}", i);

        if cfg!(feature = "generate") {
            content.push_str(" #lorem(50)");
            content = mutate(content).unwrap();
            std::fs::write("test.typ", &content).unwrap();
            {
                use std::io::Write;
                let mut f = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("mutate_sequence.log")
                    .unwrap();
                f.write_all(hex::encode(&content).as_bytes()).unwrap();
                f.write_all(b"\n").unwrap();
            }
        }

        #[cfg(not(feature = "generate"))]
        let mut incr_server = IncrDocServer::default();
        #[cfg(not(feature = "generate"))]
        let mut incr_client = IncrDocClient::default();
        #[cfg(not(feature = "generate"))]
        let mut incr_svg_client = IncrSvgDocClient::default();

        // checkout the entry file
        let main_id = driver.main_id();

        let doc = driver
            .with_shadow_file_by_id(main_id, content.as_bytes().into(), |driver| {
                driver.compile(&mut Default::default())
            })
            .unwrap();

        let delta = incr_server.pack_delta(doc.output);
        let delta = BytesModuleStream::from_slice(&delta).checkout_owned();
        incr_client.merge_delta(delta);
        incr_client.set_layout(incr_client.doc.layouts[0].unwrap_single());
        let _ = incr_svg_client.render_in_window(&mut incr_client, window);

        comemo::evict(10);
    }
}

pub fn main() {
    let workspace_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    #[cfg(feature = "generate")]
    let entry_file_path = workspace_dir.join("fuzzers/corpora/viewers/preview-incr_01.typ");
    #[cfg(not(feature = "generate"))]
    let entry_file_path = workspace_dir.join("test.typ");

    let noop_exporter = GroupExporter::new(vec![]);
    test_compiler(&workspace_dir, &entry_file_path, noop_exporter);
}
