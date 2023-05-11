use typst_ts_core::config::CompileOpts;

use crate::font::system::SystemFontSearcher;
use crate::source_manager::SourceManager;
use crate::vfs::system::SystemAccessModel;
use crate::world::CompilerFeat;
use typst_ts_core::font::FontResolverImpl;

pub type TypstSystemWorld = super::world::CompilerWorld<SystemCompilerFeat>;

pub struct SystemCompilerFeat;

impl CompilerFeat for SystemCompilerFeat {
    type M = SystemAccessModel;

    fn from_opts(opts: CompileOpts) -> (FontResolverImpl, SourceManager<Self::M>) {
        let mut searcher = SystemFontSearcher::new();
        searcher.search_system();
        searcher.add_embedded();
        for path in opts.font_paths {
            if path.is_dir() {
                searcher.search_dir(&path);
            } else {
                searcher.search_file(&path);
            }
        }
        (
            FontResolverImpl::new(searcher.book, searcher.fonts),
            SourceManager::new(SystemAccessModel {}),
        )
    }
}
