use std::borrow::Cow;

use typst_ts_core::{config::CompileOpts, error::prelude::*, font::FontResolverImpl, Bytes};

use crate::{
    font::system::SystemFontSearcher,
    package::system::SystemRegistry,
    vfs::{system::SystemAccessModel, Vfs},
};

/// type trait of [`TypstSystemWorld`].
pub struct SystemCompilerFeat;

impl crate::world::CompilerFeat for SystemCompilerFeat {
    /// It accesses a physical file system.
    type M = SystemAccessModel;
    /// It performs native HTTP requests for fetching package data.
    type R = SystemRegistry;
}

/// The compiler world in system environment.
pub type TypstSystemWorld = crate::world::CompilerWorld<SystemCompilerFeat>;

impl TypstSystemWorld {
    /// Create [`TypstSystemWorld`] with the given options.
    /// See SystemCompilerFeat for instantiation details.
    /// See [`CompileOpts`] for available options.
    pub fn new(opts: CompileOpts) -> ZResult<Self> {
        Ok(Self::new_raw(
            opts.root_dir.clone(),
            Vfs::new(SystemAccessModel {}),
            SystemRegistry::default(),
            Self::resolve_fonts(opts)?,
        ))
    }

    /// Resolve fonts from given options.
    fn resolve_fonts(opts: CompileOpts) -> ZResult<FontResolverImpl> {
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

        Ok(searcher.into())
    }
}
