/// This is an implementation for `Write + !AsRef<AnyBytes>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsWritable;

/// This is an implementation for `Vec<u8>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsOwnedBytes;

/// This is an implementation for `String`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsOwnedString;

pub use self::typst::*;
pub use reflexo::typst;
