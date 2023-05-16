//! Maps paths to compact integer ids. We don't care about clearings paths which
//! no longer exist -- the assumption is total size of paths we ever look at is
//! not too big.
use std::hash::BuildHasherDefault;
use std::hash::Hash;

use indexmap::IndexSet;
use rustc_hash::FxHasher;

use super::FileId;

/// Structure to map between [`VfsPath`] and [`FileId`].
pub(crate) struct PathInterner<P> {
    map: IndexSet<P, BuildHasherDefault<FxHasher>>,
}

impl<P> Default for PathInterner<P> {
    fn default() -> Self {
        Self {
            map: IndexSet::default(),
        }
    }
}

impl<P: Hash + Eq> PathInterner<P> {
    /// Get the id corresponding to `path`.
    ///
    /// If `path` does not exists in `self`, returns [`None`].
    pub(crate) fn get(&self, path: &P) -> Option<FileId> {
        self.map.get_index_of(path).map(|i| FileId(i as u32))
    }

    /// Insert `path` in `self`.
    ///
    /// - If `path` already exists in `self`, returns its associated id;
    /// - Else, returns a newly allocated id.
    pub(crate) fn intern(&mut self, path: P) -> FileId {
        let (id, _added) = self.map.insert_full(path);
        assert!(id < u32::MAX as usize);
        FileId(id as u32)
    }

    /// Returns the path corresponding to `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` does not exists in `self`.
    pub(crate) fn lookup(&self, id: FileId) -> &P {
        self.map.get_index(id.0 as usize).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::vfs::VfsPath;

    use super::PathInterner;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_interner_path_buf() {
        let mut interner = PathInterner::<PathBuf>::default();
        let id = interner.intern(PathBuf::from("foo"));
        assert_eq!(interner.lookup(id), &PathBuf::from("foo"));
    }

    #[test]
    fn test_interner_vfs_path() {
        let mut interner = PathInterner::<VfsPath>::default();
        let id = interner.intern(VfsPath::new_virtual_path("/foo".to_owned()));
        assert_eq!(
            interner.lookup(id),
            &VfsPath::new_virtual_path("/foo".to_owned())
        );
    }

    #[test]
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]

    fn test_interner_handle() {
        let test_path = if cfg!(target_os = "windows") {
            "C:\\Users"
        } else {
            "/usr"
        };

        let mut interner = PathInterner::<same_file::Handle>::default();
        let id = interner.intern(same_file::Handle::from_path(Path::new(test_path)).unwrap());
        assert_eq!(
            interner.lookup(id),
            &same_file::Handle::from_path(Path::new(test_path)).unwrap()
        );
    }
}
