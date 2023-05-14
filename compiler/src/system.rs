use typst_ts_core::config::CompileOpts;

use crate::font::system::SystemFontSearcher;
use crate::source_manager::SourceManager;
use crate::vfs::system::SystemAccessModel;
use crate::world::CompilerFeat;
use typst_ts_core::font::FontResolverImpl;

pub type TypstSystemWorld = crate::world::CompilerWorld<SystemCompilerFeat>;

pub struct SystemCompilerFeat;

impl CompilerFeat for SystemCompilerFeat {
    type M = SystemAccessModel;

    fn from_opts(opts: CompileOpts) -> (FontResolverImpl, SourceManager<Self::M>) {
        let mut searcher = SystemFontSearcher::new();

        if opts
            .font_profile_cache_path
            .to_str()
            .map(|e| !e.is_empty())
            .unwrap_or_default()
        {
            searcher.set_can_profile(true);
        }

        for profile_path in opts.font_profile_paths {
            searcher.add_profile_by_path(&profile_path);
        }
        if !opts.no_system_fonts {
            searcher.search_system();
        }
        searcher.add_embedded();
        for path in opts.font_paths {
            if path.is_dir() {
                searcher.search_dir(&path);
            } else {
                searcher.search_file(&path);
            }
        }
        (searcher.into(), SourceManager::new(SystemAccessModel {}))
    }
}
