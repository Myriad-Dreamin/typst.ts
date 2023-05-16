use typst_ts_compiler::font::web::BrowserFontSearcher;
pub use typst_ts_compiler::*;
use wasm_bindgen::prelude::*;

pub mod builder;

#[wasm_bindgen]
pub struct TypstCompiler {
    pub(crate) _world: TypstBrowserWorld,
}

impl TypstCompiler {
    pub async fn new(searcher: BrowserFontSearcher) -> Result<Self, JsValue> {
        Ok(Self {
            _world: TypstBrowserWorld::new_raw(
                std::path::Path::new("/").to_owned(),
                searcher.into(),
            ),
        })
    }
}
