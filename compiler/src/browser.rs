use std::path::PathBuf;

use typst_ts_core::font::FontResolverImpl;

use crate::{vfs::dummy::DummyAccessModel, world::CompilerFeat};

/// A world that provides access to the browser.

pub type TypstBrowserWorld = crate::world::CompilerWorld<BrowserCompilerFeat>;

pub struct BrowserCompilerFeat;

impl CompilerFeat for BrowserCompilerFeat {
    type M = DummyAccessModel;

    // manual construction 13MB
    // let dummy_library = typst::eval::LangItems {
    //   ...
    // };
    // typst::eval::set_lang_items(dummy_library);
}

impl TypstBrowserWorld {
    pub fn new(root_dir: PathBuf, font_resolver: FontResolverImpl) -> Self {
        let vfs = crate::vfs::Vfs::new(DummyAccessModel {});

        Self::new_raw(root_dir, vfs, font_resolver)
    }
}
