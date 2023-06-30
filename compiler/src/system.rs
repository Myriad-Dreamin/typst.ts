use std::borrow::Cow;
use std::path::PathBuf;

use typst::diag::PackageResult;
use typst::file::PackageSpec;
use typst_ts_core::{config::CompileOpts, error::prelude::*, Bytes};

use crate::font::system::SystemFontSearcher;
use crate::package::system::prepare_package;
use crate::vfs::{system::SystemAccessModel, Vfs};
use crate::world::CompilerFeat;

pub type TypstSystemWorld = crate::world::CompilerWorld<SystemCompilerFeat>;

pub struct SystemCompilerFeat;

impl CompilerFeat for SystemCompilerFeat {
    type M = SystemAccessModel;

    // todo: add package manager model
    fn resolve_package(spec: &PackageSpec) -> PackageResult<PathBuf> {
        prepare_package(spec)
    }
}

impl TypstSystemWorld {
    pub fn new(opts: CompileOpts) -> ZResult<Self> {
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

        // Note: the order of adding fonts is important.
        // See: https://github.com/typst/typst/blob/9c7f31870b4e1bf37df79ebbe1df9a56df83d878/src/font/book.rs#L151-L154
        // Source1: add the fonts specified by the user.
        for path in opts.font_paths {
            if path.is_dir() {
                searcher.search_dir(&path);
            } else {
                searcher.search_file(&path);
            }
        }
        // Source2: add the fonts in memory.
        for font_data in opts.with_embedded_fonts {
            searcher.add_memory_font(match font_data {
                Cow::Borrowed(data) => Bytes::from_static(data),
                Cow::Owned(data) => Bytes::from(data),
            });
        }
        // Source3: add the fonts from vanilla paths.
        if !opts.no_vanilla_fonts {
            searcher.search_vanilla();
        }
        // Source4: add the fonts from system paths.
        if !opts.no_system_fonts {
            searcher.search_system();
        }
        // Source5: add the fonts from the profile cache.
        for profile_path in opts.font_profile_paths {
            searcher.add_profile_by_path(&profile_path);
        }

        let font_resolver = searcher.into();

        let vfs = Vfs::new(SystemAccessModel {});

        Ok(Self::new_raw(root_dir, vfs, font_resolver))
    }
}
