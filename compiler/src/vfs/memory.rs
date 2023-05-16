use std::{path::Path, time::SystemTime};

use crate::source_manager::{AccessModel, FileMetadata};

#[derive(Default, Debug)]
pub struct MemoryAccessModel(super::model::Vfs);

pub struct MemoryFileMeta {
    mt: SystemTime,
    is_file: bool,
    src: std::path::PathBuf,
}

impl FileMetadata for MemoryFileMeta {
    type RealPath = std::path::PathBuf;

    fn mtime(&mut self) -> SystemTime {
        self.mt
    }

    fn is_file(&mut self) -> bool {
        self.is_file
    }

    fn real_path(&mut self) -> std::io::Result<Self::RealPath> {
        Ok(self.src.clone())
    }
}

impl AccessModel for MemoryAccessModel {
    type FM = MemoryFileMeta;

    fn stat(&self, src: &Path) -> std::io::Result<Self::FM> {
        Ok(MemoryFileMeta {
            mt: SystemTime::UNIX_EPOCH,
            is_file: true,
            src: src.to_owned(),
        })
    }

    fn read_all_once(&self, _src: &Path, _buf: &mut Vec<u8>) -> std::io::Result<usize> {
        panic!("Not implemented")
    }
}
