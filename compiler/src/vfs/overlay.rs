use std::sync::Arc;
use std::{collections::HashMap, path::Path};

use parking_lot::RwLock;
use typst::diag::FileResult;

use typst_ts_core::Bytes;

use crate::time::SystemTime;

use super::AccessModel;

#[derive(Debug, Clone)]
struct OverlayFileMeta {
    mt: SystemTime,
    content: Bytes,
}

#[derive(Default, Debug)]
pub struct OverlayAccessModel<M: AccessModel> {
    files: RwLock<HashMap<Arc<Path>, OverlayFileMeta>>,
    pub model: M,
}

impl<M: AccessModel> OverlayAccessModel<M> {
    pub fn new(model: M) -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            model,
        }
    }

    pub fn inner(&self) -> &M {
        &self.model
    }

    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.model
    }

    pub fn clear_shadow(&self) {
        self.files.write().clear();
    }

    pub fn file_paths(&self) -> Vec<Arc<Path>> {
        self.files.read().keys().cloned().collect()
    }

    pub fn add_file(&self, path: Arc<Path>, content: Bytes) {
        // we change mt every time, since content almost changes every time
        // Note: we can still benefit from cache, since we incrementally parse source

        let mt = SystemTime::now();
        let meta = OverlayFileMeta { mt, content };
        self.files
            .write()
            .entry(path)
            .and_modify(|e| {
                // unlikely to happen but still possible in concurrent
                // environment
                // The case is found in browser test
                if e.mt == meta.mt && e.content != meta.content {
                    e.mt = meta
                        .mt
                        // instant::SystemTime has a minimum resolution of 1ms
                        // we negate the time by 1ms so that the time is always
                        // invalidated
                        .checked_sub(std::time::Duration::from_millis(1))
                        .unwrap();
                    e.content = meta.content.clone();
                } else {
                    *e = meta.clone();
                }
            })
            .or_insert(meta);
    }

    pub fn remove_file(&self, path: &Path) {
        self.files.write().remove(path);
    }
}

impl<M: AccessModel> AccessModel for OverlayAccessModel<M> {
    type RealPath = M::RealPath;

    fn mtime(&self, src: &Path) -> FileResult<SystemTime> {
        if let Some(meta) = self.files.read().get(src) {
            return Ok(meta.mt);
        }

        self.model.mtime(src)
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        if self.files.read().get(src).is_some() {
            return Ok(true);
        }

        self.model.is_file(src)
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        if self.files.read().get(src).is_some() {
            return Ok(src.into());
        }

        self.model.real_path(src)
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        if let Some(meta) = self.files.read().get(src) {
            return Ok(meta.content.clone());
        }

        self.model.content(src)
    }
}
