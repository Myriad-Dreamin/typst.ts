use std::borrow::Cow;

use typst::util::Buffer;
use typst_ts_core::config::CompileOpts;

use crate::font::system::SystemFontSearcher;
use crate::vfs::{system::SystemAccessModel, Vfs};
use crate::world::CompilerFeat;

pub type TypstSystemWorld = crate::world::CompilerWorld<SystemCompilerFeat>;

pub struct SystemCompilerFeat;

impl CompilerFeat for SystemCompilerFeat {
    type M = SystemAccessModel;
}

impl TypstSystemWorld {
    pub fn new(opts: CompileOpts) -> Self {
        let root_dir = opts.root_dir.clone();
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
        for path in opts.font_paths {
            if path.is_dir() {
                searcher.search_dir(&path);
            } else {
                searcher.search_file(&path);
            }
        }
        if !opts.no_vanilla_fonts {
            searcher.search_vanilla();
        }
        for font_data in opts.with_embedded_fonts {
            searcher.add_memory_font(match font_data {
                Cow::Borrowed(data) => Buffer::from_static(data),
                Cow::Owned(data) => Buffer::from(data),
            });
        }
        let font_resolver = searcher.into();

        let vfs = Vfs::new(SystemAccessModel {});

        Self::new_raw(root_dir, vfs, font_resolver)
    }
}
