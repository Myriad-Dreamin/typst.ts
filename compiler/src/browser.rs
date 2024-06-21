use std::{path::PathBuf, sync::Arc};

use comemo::Prehashed;
use parking_lot::RwLock;
use typst_ts_core::{config::compiler::EntryState, font::FontResolverImpl, TypstDict};

use crate::{package::browser::ProxyRegistry, vfs::browser::ProxyAccessModel};

/// A world that provides access to the browser.
/// It is under development.
pub type TypstBrowserUniverse = crate::world::CompilerUniverse<BrowserCompilerFeat>;
pub type TypstBrowserWorld = crate::world::CompilerWorld<BrowserCompilerFeat>;

#[derive(Debug, Clone, Copy)]
pub struct BrowserCompilerFeat;

impl crate::world::CompilerFeat for BrowserCompilerFeat {
    /// Uses [`FontResolverImpl`] directly.
    type FontResolver = FontResolverImpl;
    type AccessModel = ProxyAccessModel;
    type Registry = ProxyRegistry;

    // manual construction 13MB
    // let dummy_library = typst::eval::LangItems {
    //   ...
    // };
    // typst::eval::set_lang_items(dummy_library);
}

// todo
unsafe impl Send for ProxyRegistry {}
unsafe impl Sync for ProxyRegistry {}

impl TypstBrowserUniverse {
    pub fn new(
        root_dir: PathBuf,
        inputs: Option<Arc<Prehashed<TypstDict>>>,
        access_model: ProxyAccessModel,
        registry: ProxyRegistry,
        font_resolver: FontResolverImpl,
    ) -> Self {
        let vfs = crate::vfs::Vfs::new(access_model);

        Self::new_raw(
            EntryState::new_rooted(root_dir.into(), None),
            inputs,
            Arc::new(RwLock::new(vfs)),
            registry,
            Arc::new(font_resolver),
        )
    }
}
