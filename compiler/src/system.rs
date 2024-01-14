use typst_ts_core::{config::CompileOpts, error::prelude::*, font::FontResolverImpl};

use crate::{
    font::system::SystemFontSearcher,
    package::http::HttpRegistry,
    vfs::{system::SystemAccessModel, Vfs},
};

/// type trait of [`TypstSystemWorld`].
#[derive(Debug, Clone, Copy)]
pub struct SystemCompilerFeat;

impl crate::world::CompilerFeat for SystemCompilerFeat {
    /// It accesses a physical file system.
    type AccessModel = SystemAccessModel;
    /// It performs native HTTP requests for fetching package data.
    type Registry = HttpRegistry;
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
            HttpRegistry::default(),
            Self::resolve_fonts(opts)?,
        ))
    }

    /// Resolve fonts from given options.
    fn resolve_fonts(opts: CompileOpts) -> ZResult<FontResolverImpl> {
        let mut searcher = SystemFontSearcher::new();
        searcher.resolve_opts(opts)?;
        Ok(searcher.into())
    }
}
