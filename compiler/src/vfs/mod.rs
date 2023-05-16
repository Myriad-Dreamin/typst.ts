#[cfg(feature = "system")]
pub mod system;

pub mod memory;

pub(crate) mod model;
pub use model::{file_set::FileSetConfigBuilder, AbsPath, AbsPathBuf, Vfs as MemVfs, VfsPath};
