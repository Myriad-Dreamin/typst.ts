use std::path::PathBuf;

use typst_ts_core::font::FontResolverImpl;

use crate::{package::dummy::DummyRegistry, vfs::browser::ProxyAccessModel};

/// A world that provides access to the browser.
/// It is under development.
pub type TypstBrowserWorld = crate::world::CompilerWorld<BrowserCompilerFeat>;

pub struct BrowserCompilerFeat;

impl crate::world::CompilerFeat for BrowserCompilerFeat {
    type AccessModel = ProxyAccessModel;
    type Registry = DummyRegistry;

    // manual construction 13MB
    // let dummy_library = typst::eval::LangItems {
    //   ...
    // };
    // typst::eval::set_lang_items(dummy_library);
}

impl TypstBrowserWorld {
    pub fn new(
        root_dir: PathBuf,
        access_model: ProxyAccessModel,
        font_resolver: FontResolverImpl,
    ) -> Self {
        let vfs = crate::vfs::Vfs::new(access_model);

        Self::new_raw(root_dir, vfs, DummyRegistry::default(), font_resolver)
    }
}
