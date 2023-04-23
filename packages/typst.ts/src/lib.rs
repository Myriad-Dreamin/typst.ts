pub mod browser_world;

pub(crate) mod pixmap;

pub(crate) mod renderer;
pub use renderer::{TypstRenderer, TypstRendererBuilder};

#[macro_use]
pub(crate) mod utils;

pub mod web_font;

#[cfg(test)]
mod tests {
    use typst::util::Buffer;

    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_render_document() {
        let mut root_path = PathBuf::new();
        root_path.push(".");

        let mut builder = TypstRendererBuilder::new().unwrap();

        // todo: prepare font files for test
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_R.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RB.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RBI.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RI.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/NewCMMath-Book.otf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/NewCMMath-Regular.otf"
        )));
        let renderer = pollster::block_on(builder.build()).unwrap();

        let path = Path::new("fuzzers/corpora/hw/main.artifact.json");
        let artifact_content = std::fs::read_to_string(path).unwrap();

        let mut ses = renderer.parse_artifact(artifact_content).unwrap();
        ses.pixel_per_pt = 2.;
        ses.background_color = "ffffff".to_string();

        renderer.render_to_image_internal(&ses).unwrap();
    }
}
