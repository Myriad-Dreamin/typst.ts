use std::{path::Path, sync::atomic::AtomicU64};

use typst::diag::FileResult;

use typst_ts_core::Bytes;

use crate::time::SystemTime;

use super::{cached::CachedAccessModel, AccessModel};

pub struct TraceAccessModel<M: AccessModel + Sized> {
    inner: M,
    trace: [AtomicU64; 6],
}

impl<M: AccessModel + Sized, C: Clone> TraceAccessModel<CachedAccessModel<M, C>> {
    pub fn new(inner: CachedAccessModel<M, C>) -> Self {
        Self {
            inner,
            trace: Default::default(),
        }
    }

    pub fn inner(&self) -> &M {
        self.inner.inner()
    }

    pub fn inner_mut(&mut self) -> &mut M {
        self.inner.inner_mut()
    }

    pub fn read_all_diff(
        &self,
        src: &Path,
        compute: impl FnOnce(Option<C>, String) -> FileResult<C>,
    ) -> FileResult<C> {
        let instant = instant::Instant::now();
        let res = self.inner.read_all_diff(src, compute);
        let elapsed = instant.elapsed();
        self.trace[4].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        crate::utils::console_log!("read_all_diff: {:?} {:?}", src, elapsed);
        res
    }
}

impl<M: AccessModel + Sized> AccessModel for TraceAccessModel<M> {
    fn clear(&mut self) {
        self.inner.clear();
    }

    fn mtime(&self, src: &Path) -> FileResult<SystemTime> {
        let instant = instant::Instant::now();
        let res = self.inner.mtime(src);
        let elapsed = instant.elapsed();
        // self.trace[0] += elapsed.as_nanos() as u64;
        self.trace[0].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        crate::utils::console_log!("mtime: {:?} {:?} => {:?}", src, elapsed, res);
        res
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        let instant = instant::Instant::now();
        let res = self.inner.is_file(src);
        let elapsed = instant.elapsed();
        self.trace[1].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        crate::utils::console_log!("is_file: {:?} {:?}", src, elapsed);
        res
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        let instant = instant::Instant::now();
        let res = self.inner.real_path(src);
        let elapsed = instant.elapsed();
        self.trace[2].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        crate::utils::console_log!("real_path: {:?} {:?}", src, elapsed);
        res
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        let instant = instant::Instant::now();
        let res = self.inner.content(src);
        let elapsed = instant.elapsed();
        self.trace[3].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        crate::utils::console_log!("read_all: {:?} {:?}", src, elapsed);
        res
    }

    type RealPath = M::RealPath;
}
