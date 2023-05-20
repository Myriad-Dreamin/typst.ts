use std::{path::Path, time::SystemTime};

use typst::{
    diag::{FileError, FileResult},
    util::Buffer,
};

use super::AccessModel;

#[derive(Default, Debug)]
pub struct DummyAccessModel;

impl AccessModel for DummyAccessModel {
    type RealPath = std::path::PathBuf;

    fn mtime(&self, _src: &Path) -> FileResult<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    fn is_file(&self, _src: &Path) -> FileResult<bool> {
        Ok(true)
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        Ok(src.to_owned())
    }

    fn read_all(&self, _src: &Path) -> FileResult<Buffer> {
        Err(FileError::AccessDenied)
    }
}
