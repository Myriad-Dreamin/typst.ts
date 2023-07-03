use std::{path::Path, sync::atomic::AtomicU64};

use typst::diag::FileResult;

use typst_ts_core::Bytes;

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

    pub fn read_all_diff(
        &self,
        src: &Path,
        compute: impl FnOnce(Option<C>, String) -> FileResult<C>,
    ) -> FileResult<C> {
        let instant = std::time::Instant::now();
        let res = self.inner.read_all_diff(src, compute);
        let elapsed = instant.elapsed();
        self.trace[4].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        println!("read_all_diff: {:?} {:?}", src, elapsed);
        res
    }
}

impl<M: AccessModel + Sized> AccessModel for TraceAccessModel<M> {
    fn clear(&mut self) {
        self.inner.clear();
    }

    fn mtime(&self, src: &Path) -> FileResult<std::time::SystemTime> {
        let instant = std::time::Instant::now();
        let res = self.inner.mtime(src);
        let elapsed = instant.elapsed();
        // self.trace[0] += elapsed.as_nanos() as u64;
        self.trace[0].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        println!("mtime: {:?} {:?}", src, elapsed);
        res
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        let instant = std::time::Instant::now();
        let res = self.inner.is_file(src);
        let elapsed = instant.elapsed();
        self.trace[1].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        println!("is_file: {:?} {:?}", src, elapsed);
        res
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        let instant = std::time::Instant::now();
        let res = self.inner.real_path(src);
        let elapsed = instant.elapsed();
        self.trace[2].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        println!("real_path: {:?} {:?}", src, elapsed);
        res
    }

    fn read_all(&self, src: &Path) -> FileResult<Bytes> {
        let instant = std::time::Instant::now();
        let res = self.inner.read_all(src);
        let elapsed = instant.elapsed();
        self.trace[3].fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        println!("read_all: {:?} {:?}", src, elapsed);
        res
    }

    type RealPath = M::RealPath;
}
