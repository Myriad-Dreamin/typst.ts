use typst_ts_core::{config::CompileOpts, font::FontResolverImpl};

use crate::{
    vfs::{memory::MemoryAccessModel, Vfs},
    world::CompilerFeat,
};

/// A world that provides access to the browser.

pub type TypstBrowserWorld = crate::world::CompilerWorld<BrowserCompilerFeat>;

pub struct BrowserCompilerFeat;

impl CompilerFeat for BrowserCompilerFeat {
    type M = MemoryAccessModel;

    // manual construction 13MB
    // let dummy_library = typst::eval::LangItems {
    //   ...
    // };
    // typst::eval::set_lang_items(dummy_library);

    fn create_vfs() -> Vfs<Self::M> {
        Vfs::new(MemoryAccessModel::default())
    }

    fn from_opts(_opts: CompileOpts) -> (FontResolverImpl,) {
        panic!("unimplemented")
    }
}
