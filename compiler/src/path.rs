use once_cell::sync::OnceCell;
use same_file::Handle;
use siphasher::sip128::{Hasher128, SipHasher};
use std::{hash::Hash, path::Path};
use typst::diag::{FileError, FileResult};
use typst::syntax::SourceId;
use typst::util::Buffer;

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Default)]
pub struct PathSlot {
    pub source: OnceCell<FileResult<SourceId>>,
    pub buffer: OnceCell<FileResult<Buffer>>,
}

/// A hash that is the same for all paths pointing to the same entity.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PathHash(u128);

impl PathHash {
    pub fn new(path: &Path) -> FileResult<Self> {
        let f = |e| FileError::from_io(e, path);
        let handle = Handle::from_path(path).map_err(f)?;
        let mut state = SipHasher::new();
        handle.hash(&mut state);
        Ok(Self(state.finish128().as_u128()))
    }
}
