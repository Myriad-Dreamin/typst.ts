use std::path::Path;

use typst::diag::{FileError, FileResult};

use typst_ts_core::Bytes;

use crate::Time;

use super::AccessModel;

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
