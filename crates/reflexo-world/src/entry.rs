use std::path::Path;
use std::sync::Arc;

use typst::diag::SourceResult;
use typst::syntax::FileId;

pub trait EntryReader {
    fn entry_state(&self) -> EntryState;

    fn workspace_root(&self) -> Option<Arc<Path>> {
        self.entry_state().root().clone()
    }

    fn main_id(&self) -> Option<FileId> {
        self.entry_state().main()
    }
}

pub trait EntryManager: EntryReader {
    fn reset(&mut self) -> SourceResult<()> {
        Ok(())
    }

    fn mutate_entry(&mut self, state: EntryState) -> SourceResult<EntryState>;
}

pub use typst_ts_core::config::compiler::*;
