use std::{fs::File, io::Read};

use typst_ts_core::ReadAllOnce;

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
