use std::{collections::HashMap, ffi::OsStr, hash::Hash, path::Path, sync::Arc};

use append_only_vec::AppendOnlyVec;
use parking_lot::{Mutex, RwLock, RwLockUpgradableReadGuard};
use typst::{
    diag::{FileError, FileResult},
    syntax::{Source, SourceId},
    util::{Buffer, PathExt},
};

pub trait AccessModel {
    type RealPath: Hash + Eq + PartialEq;

    fn mtime(&self, src: &Path) -> std::io::Result<std::time::SystemTime>;

    fn is_file(&self, src: &Path) -> std::io::Result<bool>;

    fn real_path(&self, src: &Path) -> std::io::Result<Self::RealPath>;

    fn read_all(&self, src: &Path, buf: &mut Vec<u8>) -> std::io::Result<usize>;
}

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Default)]
pub struct PathSlot {
    source: Option<FileResult<SourceId>>,
    buffer: Option<FileResult<Buffer>>,
}

pub type PathSlotRef = Arc<RwLock<PathSlot>>;

pub struct SourceManager<M: AccessModel + Sized> {
    access_model: M,

    path2slot: RwLock<HashMap<Arc<OsStr>, PathSlotRef>>,
    key2slot: Mutex<HashMap<<M as AccessModel>::RealPath, PathSlotRef>>,
    pub sources: AppendOnlyVec<Source>,
}

impl<M: AccessModel + Sized> SourceManager<M> {
    pub fn new(access_model: M) -> Self {
        Self {
            access_model,
            sources: AppendOnlyVec::new(),
            path2slot: RwLock::new(HashMap::new()),
            key2slot: Mutex::new(HashMap::new()),
        }
    }

    /// Read a file.
    fn read(&self, path: &Path) -> FileResult<Vec<u8>> {
        let f = |e| FileError::from_io(e, path);
        if self.access_model.is_file(path).map_err(f)? {
            let mut data = vec![];
            self.access_model.read_all(path, &mut data).map_err(f)?;
            Ok(data)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    fn slot(&self, origin_path: &Path) -> FileResult<PathSlotRef> {
        let f = |e| FileError::from_io(e, origin_path);

        let path2slot = self.path2slot.upgradable_read();
        if let Some(slot) = path2slot.get(origin_path.as_os_str()) {
            return Ok(slot.clone());
        }

        let real_path = self.access_model.real_path(origin_path).map_err(f)?;

        let slot = {
            let mut key2slot = self.key2slot.lock();
            key2slot.entry(real_path).or_default().clone()
        };

        let mut path2slot = RwLockUpgradableReadGuard::upgrade(path2slot);

        let inserted = path2slot.insert(origin_path.as_os_str().into(), slot.clone());
        assert!(matches!(inserted, None), "slot already inserted");

        drop(path2slot);
        Ok(slot)
    }

    pub fn source(&self, id: SourceId) -> &Source {
        &self.sources[id.into_u16() as usize]
    }

    pub fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        let slot = self.slot(path)?;
        let slot = slot.upgradable_read();

        if let Some(ref s) = slot.source {
            return s.clone();
        }

        let mut slot = RwLockUpgradableReadGuard::upgrade(slot);
        let buf = self.read(path)?;
        let text = String::from_utf8(buf)?;

        let res = Ok(self.insert(path, text));
        slot.source = Some(res.clone());
        res
    }

    // todo: remove
    pub fn resolve_with<P: AsRef<Path>>(&self, path: P, content: &str) -> FileResult<SourceId> {
        let slot = self.slot(path.as_ref())?;
        let slot = slot.upgradable_read();

        if let Some(ref s) = slot.source {
            return s.clone();
        }

        let mut slot = RwLockUpgradableReadGuard::upgrade(slot);

        let res = Ok(self.insert(path, content.to_owned()));
        slot.source = Some(res.clone());
        res
    }

    pub fn file(&self, path: &Path) -> FileResult<Buffer> {
        let slot = self.slot(path)?;
        let slot = slot.upgradable_read();

        if let Some(ref s) = slot.buffer {
            return s.clone();
        }

        let mut slot = RwLockUpgradableReadGuard::upgrade(slot);
        let buf = self.read(path)?;
        let buf = Buffer::from(buf);

        let res = Ok(buf);
        slot.buffer = Some(res.clone());
        res
    }

    pub fn insert<P: AsRef<Path>>(&self, path: P, text: String) -> SourceId {
        let path = path.as_ref();

        let id = SourceId::from_u16(self.sources.len() as u16);
        let source = Source::new(id, path, text);
        self.sources.push(source);
        id
    }

    pub fn dependant<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref().normalize();
        self.path2slot.read().contains_key(path.as_os_str())
    }

    pub fn reset(&mut self) {
        self.sources = AppendOnlyVec::new();
        self.path2slot.get_mut().clear();
        self.key2slot.get_mut().clear();
    }
}
