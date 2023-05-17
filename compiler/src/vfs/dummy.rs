use std::{path::Path, time::SystemTime};

use super::AccessModel;

#[derive(Default, Debug)]
pub struct DummyAccessModel;

impl AccessModel for DummyAccessModel {
    type RealPath = std::path::PathBuf;

    fn mtime(&self, _src: &Path) -> std::io::Result<SystemTime> {
        Ok(SystemTime::UNIX_EPOCH)
    }

    fn is_file(&self, _src: &Path) -> std::io::Result<bool> {
        Ok(true)
    }

    fn real_path(&self, src: &Path) -> std::io::Result<Self::RealPath> {
        Ok(src.to_owned())
    }

    fn read_all(&self, _src: &Path, _buf: &mut Vec<u8>) -> std::io::Result<usize> {
        panic!("Not implemented")
    }
}
