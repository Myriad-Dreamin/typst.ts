use std::path::Path;

use typst::diag::{FileError, FileResult};

use typst_ts_core::Bytes;

use crate::Time;

use super::AccessModel;

/// Provides dummy access model.
///
/// Note: we can still perform compilation with dummy access model, since
/// [`super::Vfs`] will make a overlay access model over the provided dummy
/// access model.
#[derive(Default, Debug, Clone, Copy)]
pub struct DummyAccessModel;

impl AccessModel for DummyAccessModel {
    type RealPath = std::path::PathBuf;

    fn mtime(&self, _src: &Path) -> FileResult<Time> {
        Ok(Time::UNIX_EPOCH)
    }

    fn is_file(&self, _src: &Path) -> FileResult<bool> {
        Ok(true)
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        Ok(src.to_owned())
    }

    fn content(&self, _src: &Path) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }
}
