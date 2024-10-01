#[macro_use]
pub(crate) mod utils;
pub(crate) mod builder;
pub(crate) mod render;
pub(crate) mod session;
#[cfg(feature = "worker")]
pub(crate) mod worker;

pub use builder::TypstRendererBuilder;
pub use session::RenderSession;
pub use session::RenderSessionOptions;

use reflexo_typst::error::prelude::*;
#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use session::CreateSessionOptions;

pub mod build_info {
    /// The version of the typst-ts-renderer crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");

    /// The features of the typst-ts-renderer crate.
    pub static FEATURES: &str = env!("VERGEN_CARGO_FEATURES");

    /// The commit hash of the typst-ts-renderer crate.
    pub static COMMIT_HASH: &str = env!("VERGEN_GIT_SHA");

    /// The profile.opt_level of the typst-ts-renderer crate.
    pub static OPT_LEVEL: &str = env!("VERGEN_CARGO_OPT_LEVEL");

    pub fn features() -> Vec<&'static str> {
        FEATURES.split(',').collect::<Vec<_>>()
    }
}

/// Return an object containing build info
/// CodeSize: 4KB
#[wasm_bindgen]
pub fn renderer_build_info() -> JsValue {
    let obj = js_sys::Object::new();

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("version"),
        &JsValue::from_str(build_info::VERSION),
    )
    .unwrap();

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("features"),
        &build_info::features()
            .into_iter()
            .map(JsValue::from_str)
            .collect::<js_sys::Array>(),
    )
    .unwrap();

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("commit_hash"),
        &JsValue::from_str(build_info::COMMIT_HASH),
    )
    .unwrap();

    js_sys::Reflect::set(
        &obj,
        &JsValue::from_str("profile"),
        &JsValue::from_str(&format!(
            "opt_level({opt_level})",
            opt_level = build_info::OPT_LEVEL
        )),
    )
    .unwrap();

    obj.into()
}

#[wasm_bindgen]
#[derive(Debug, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, Serialize, Deserialize))]
pub struct RenderPageImageOptions {
    /// pixel per point
    pub(crate) pixel_per_pt: Option<f32>,
    /// background color
    pub(crate) background_color: Option<String>,

    pub(crate) page_off: usize,
    pub(crate) cache_key: Option<String>,
    pub(crate) data_selection: Option<u32>,
}

#[wasm_bindgen]
impl RenderPageImageOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            pixel_per_pt: None,
            background_color: None,
            page_off: 0,
            cache_key: None,
            data_selection: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> Option<f32> {
        self.pixel_per_pt
    }

    #[wasm_bindgen(setter)]
    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: Option<f32>) {
        self.pixel_per_pt = pixel_per_pt;
    }

    #[wasm_bindgen(getter)]
    pub fn background_color(&self) -> Option<String> {
        self.background_color.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_background_color(&mut self, background_color: Option<String>) {
        self.background_color = background_color;
    }

    #[wasm_bindgen(getter)]
    pub fn page_off(&self) -> usize {
        self.page_off
    }

    #[wasm_bindgen(setter)]
    pub fn set_page_off(&mut self, page_off: usize) {
        self.page_off = page_off;
    }

    #[wasm_bindgen(getter)]
    pub fn cache_key(&self) -> Option<String> {
        self.cache_key.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_cache_key(&mut self, cache_key: Option<String>) {
        self.cache_key = cache_key;
    }

    #[wasm_bindgen(getter)]
    pub fn data_selection(&self) -> Option<u32> {
        self.data_selection
    }

    #[wasm_bindgen(setter)]
    pub fn set_data_selection(&mut self, data_selection: Option<u32>) {
        self.data_selection = data_selection;
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct TypstRenderer {}

impl Default for TypstRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl TypstRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TypstRenderer {
        Self {}
    }

    pub fn create_session(&self, options: Option<CreateSessionOptions>) -> ZResult<RenderSession> {
        match options {
            Some(options) => {
                let format = options.format.as_deref().unwrap_or("vector");

                let artifact_content = options
                    .artifact_content
                    .as_deref()
                    .ok_or_else(|| error_once!("Renderer.MissingArtifactContent"))?;

                self.session_from_artifact(artifact_content, format)
            }
            None => Ok(RenderSession::default()),
        }
    }

    pub fn reset(&mut self, session: &mut RenderSession) -> ZResult<()> {
        session.reset();
        Ok(())
    }

    pub fn manipulate_data(
        &mut self,
        session: &mut RenderSession,
        action: &str,
        data: &[u8],
    ) -> ZResult<()> {
        match action {
            "reset" => session.reset_current(data),
            "merge" => session.merge_delta(data),
            _ => Err(error_once!("Renderer.UnsupportedAction", action: action)),
        }
    }

    pub fn session_from_artifact(
        &self,
        artifact_content: &[u8],
        decoder: &str,
    ) -> ZResult<RenderSession> {
        if decoder == "vector" {
            return self.session_from_vector_artifact(artifact_content);
        }

        if decoder == "serde_json" || decoder == "js" || decoder == "ir" {
            Err(error_once!("deprecated format are removal in v0.4.0"))?
        }

        Err(error_once!("Renderer.UnsupportedDecoder", decoder: decoder))
    }

    fn session_from_vector_artifact(&self, artifact_content: &[u8]) -> ZResult<RenderSession> {
        let mut session = RenderSession::default();
        session.reset_current(artifact_content)?;
        Ok(session)
    }

    // ses.pixel_per_pt = options.as_ref().and_then(|o|
    // o.pixel_per_pt).unwrap_or(2.);

    // ses.background_color = options
    //     .as_ref()
    //     .and_then(|o| o.background_color.clone())
    //     .unwrap_or("ffffff".to_string());
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::{TypstRenderer, TypstRendererBuilder};
    use std::path::PathBuf;

    pub fn get_renderer() -> TypstRenderer {
        let mut root_path = PathBuf::new();
        root_path.push(".");

        let builder = TypstRendererBuilder::new().unwrap();

        pollster::block_on(builder.build()).unwrap()
    }

    // todo: export svg image
    #[test]
    #[cfg(feature = "test_render_document")]
    fn test_render_document() {
        fn artifact_path() -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fuzzers/corpora/math/main.artifact.json")
        }

        let renderer = get_renderer();

        let artifact_content = std::fs::read(artifact_path()).unwrap();

        let mut ses = renderer
            .session_mgr
            .session_from_artifact(artifact_content.as_slice(), "serde_json")
            .unwrap();
        ses.pixel_per_pt = 2.;
        ses.background_color = "ffffff".to_string();

        renderer.render_to_image_internal(&ses, None).unwrap();
    }
}
