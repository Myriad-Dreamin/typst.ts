use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use typst::diag::{FileError, FileResult};

use typst_ts_core::{Bytes, ReadAllOnce};

use super::AccessModel;

pub struct LazyFile {
    path: std::path::PathBuf,
    file: Option<std::io::Result<File>>,
}

impl LazyFile {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path, file: None }
    }
}

impl ReadAllOnce for LazyFile {
    fn read_all(mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let file = self.file.get_or_insert_with(|| File::open(&self.path));
        let Ok(ref mut file) = file else {
            let err = file.as_ref().unwrap_err();
            // todo: clone error or hide error
            return Err(std::io::Error::new(err.kind(), err.to_string()));
        };

        file.read_to_end(buf)
    }
}

pub struct SystemFileMeta {
    mt: std::time::SystemTime,
    is_file: bool,
}

pub struct SystemAccessModel;

impl SystemAccessModel {
    fn stat(&self, src: &Path) -> std::io::Result<SystemFileMeta> {
        let meta = std::fs::metadata(src)?;
        Ok(SystemFileMeta {
            mt: meta.modified()?,
            is_file: meta.is_file(),
        })
    }
}

impl AccessModel for SystemAccessModel {
    type RealPath = PathBuf; // same_file::Handle;

    fn mtime(&self, src: &Path) -> FileResult<std::time::SystemTime> {
        let f = |e| FileError::from_io(e, src);
        Ok(self.stat(src).map_err(f)?.mt)
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        let f = |e| FileError::from_io(e, src);
        Ok(self.stat(src).map_err(f)?.is_file)
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        // let f = |e| FileError::from_io(e, src);
        // same_file::Handle::from_path(src).map_err(f)
        Ok(src.to_path_buf())
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        let f = |e| FileError::from_io(e, src);
        let mut buf = Vec::<u8>::new();
        std::fs::File::open(src)
            .map_err(f)?
            .read_to_end(&mut buf)
            .map_err(f)?;
        Ok(buf.into())
    }
}
