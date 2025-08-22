use std::path::Path;

use reflexo_typst::config::{entry::EntryOpts, CompileOpts};
use reflexo_typst::vector::{
    ir::{Abs, Point, Rect},
    stream::BytesModuleStream,
};
use reflexo_typst::{Bytes, TypstDocument, TypstSystemUniverse};
use reflexo_typst2vec::incr::{IncrDocClient, IncrDocServer};
use reflexo_vec2svg::IncrSvgDocClient;

fn get_driver(workspace_dir: &Path, entry_file_path: &Path) -> TypstSystemUniverse {
    let project_base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let w = project_base.join("fonts");
    let font_path = project_base.join("assets/fonts");
    let verse = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.into()),
        no_system_fonts: true,
        font_paths: vec![w, font_path],
        ..CompileOpts::default()
    })
    .unwrap();

    verse.with_entry_file(entry_file_path.to_owned())
}

pub fn test_compiler(workspace_dir: &Path, entry_file_path: &Path) {
    let driver = get_driver(workspace_dir, entry_file_path);
    let mut content = { std::fs::read_to_string(entry_file_path).expect("Could not read file") };

    let mut incr_server = IncrDocServer::default();
    let mut incr_client = IncrDocClient::default();
    let mut incr_svg_client = IncrSvgDocClient::default();

    let window = Rect {
        lo: Point::new(Abs::from(0.), Abs::from(0.)),
        hi: Point::new(Abs::from(1e33), Abs::from(1e33)),
    };
    let _ = incr_svg_client.render_in_window(&mut incr_client, window);

    let mut diff = vec![];

    // checkout the entry file

    let doc = driver
        .snapshot_with_entry_content(Bytes::from_string(content.clone()), None)
        .compile()
        .output
        .unwrap();
    let server_delta = incr_server.pack_delta(&TypstDocument::Paged(doc));
    let server_delta = BytesModuleStream::from_slice(&server_delta).checkout_owned();
    incr_client.merge_delta(server_delta);
    let _ = incr_svg_client.render_in_window(&mut incr_client, window);

    for i in 0..20 {
        eprintln!("Iteration {i}");

        // content = content.replace("@netwok2020", "@netwok2020 x");
        content += "\n\nx";

        let doc = driver
            .snapshot_with_entry_content(Bytes::from_string(content.clone()), None)
            .compile()
            .output
            .unwrap();

        let server_delta = incr_server.pack_delta(&TypstDocument::Paged(doc));
        let sd = server_delta.len();
        let server_delta = BytesModuleStream::from_slice(&server_delta).checkout_owned();
        incr_client.merge_delta(server_delta);
        incr_client.set_layout(incr_client.doc.layouts[0].unwrap_single());
        let cd = incr_svg_client.render_in_window(&mut incr_client, window);
        // std::fs::write(format!("{}.svg", i), cd.clone()).unwrap();
        diff.push((sd, cd.len()));

        comemo::evict(10);
    }

    eprintln!("diff: {diff:?}");
}

pub fn main() {
    // #[cfg(feature = "ieee")]
    let workspace_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    // #[cfg(feature = "ieee")]
    let entry_file_path = workspace_dir.join("fuzzers/corpora/typst-templates/ieee/main.typ");

    #[cfg(feature = "pku-thesis")]
    let workspace_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../ts/pkuthss-typst/");
    #[cfg(feature = "pku-thesis")]
    let entry_file_path = workspace_dir.join(r#"thesis.typ"#);

    // let workspace_dir =
    //     std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../typst/
    // masterproef/"); let entry_file_path =
    // workspace_dir.join(r#"masterproef/main.typ"#);

    for i in 0..10 {
        eprintln!("Over Iteration {i}");
        test_compiler(&workspace_dir, &entry_file_path);
    }
}
