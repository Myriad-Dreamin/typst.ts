use std::{fs::File, io::Read, path::Path};

use typst_ts_core::ReadAllOnce;

use crate::source_manager::{AccessModel, FileMetadata};

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
    src: std::path::PathBuf,
}

impl FileMetadata for SystemFileMeta {
    type RealPath = same_file::Handle;

    fn mtime(&mut self) -> std::time::SystemTime {
        self.mt
    }

    fn is_file(&mut self) -> bool {
        self.is_file
    }

    fn real_path(&mut self) -> std::io::Result<Self::RealPath> {
        same_file::Handle::from_path(&self.src)
    }
}

pub struct SystemAccessModel;

impl AccessModel for SystemAccessModel {
    type FM = SystemFileMeta;

    fn stat(&self, src: &Path) -> std::io::Result<Self::FM> {
        let meta = std::fs::metadata(src)?;
        Ok(SystemFileMeta {
            mt: meta.modified()?,
            is_file: meta.is_file(),
            src: src.to_owned(),
        })
    }

    fn read_all_once(&self, src: &Path, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        std::fs::File::open(src)?.read_to_end(buf)
    }
}
