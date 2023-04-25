pub(crate) mod browser_world;

pub(crate) mod pixmap;

pub(crate) mod renderer;
pub use renderer::session::{RenderSessionManager, RenderSessionOptions};
pub use renderer::{TypstRenderer, TypstRendererBuilder};

#[macro_use]
pub(crate) mod utils;

pub mod web_font;

#[cfg(test)]
mod tests {
    use typst::util::Buffer;

    use super::renderer::TypstRendererBuilder;
    use std::path::PathBuf;

    fn artifact_path() -> PathBuf {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fuzzers/corpora/math/math.artifact.json");
        path
    }

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

        let artifact_content = std::fs::read(artifact_path()).unwrap();

        let mut ses = renderer
            .session_from_artifact_internal(artifact_content.as_slice())
            .unwrap();
        ses.pixel_per_pt = 2.;
        ses.background_color = "ffffff".to_string();

        renderer.render_to_image_internal(&ses, None).unwrap();
    }
}
