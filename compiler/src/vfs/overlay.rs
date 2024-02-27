use std::sync::Arc;
use std::{collections::HashMap, path::Path};

use parking_lot::RwLock;
use typst::diag::FileResult;

use typst_ts_core::Bytes;

use crate::Time;

use super::AccessModel;

#[derive(Debug, Clone)]
struct OverlayFileMeta {
    mt: Time,
    content: Bytes,
}

/// Provides overlay access model which allows to shadow the underlying access
/// model with memory contents.
#[derive(Default, Debug)]
pub struct OverlayAccessModel<M: AccessModel> {
    files: RwLock<HashMap<Arc<Path>, OverlayFileMeta>>,
    /// The underlying access model
    pub inner: M,
}

impl<M: AccessModel> OverlayAccessModel<M> {
    /// Create a new [`OverlayAccessModel`] with the given inner access model
    pub fn new(inner: M) -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            inner,
        }
    }

    /// Get the inner access model
    pub fn inner(&self) -> &M {
        &self.inner
    }

    /// Get the mutable reference to the inner access model
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.inner
    }

    /// Clear the shadowed files
    pub fn clear_shadow(&self) {
        self.files.write().clear();
    }

    /// Get the shadowed file paths
    pub fn file_paths(&self) -> Vec<Arc<Path>> {
        self.files.read().keys().cloned().collect()
    }

    /// Add a shadow file to the [`OverlayAccessModel`]
    pub fn add_file(&self, path: Arc<Path>, content: Bytes) {
        // we change mt every time, since content almost changes every time
        // Note: we can still benefit from cache, since we incrementally parse source

        let mt = crate::time::now();
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
                        // [`crate::Time`] has a minimum resolution of 1ms
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

    /// Remove a shadow file from the [`OverlayAccessModel`]
    pub fn remove_file(&self, path: &Path) {
        self.files.write().remove(path);
    }
}

impl<M: AccessModel> AccessModel for OverlayAccessModel<M> {
    type RealPath = M::RealPath;

    fn mtime(&self, src: &Path) -> FileResult<Time> {
        if let Some(meta) = self.files.read().get(src) {
            return Ok(meta.mt);
        }

        self.inner.mtime(src)
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        if self.files.read().get(src).is_some() {
            return Ok(true);
        }

        self.inner.is_file(src)
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        if self.files.read().get(src).is_some() {
            return Ok(src.into());
        }

        self.inner.real_path(src)
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        if let Some(meta) = self.files.read().get(src) {
            return Ok(meta.content.clone());
        }

        self.inner.content(src)
    }
}
