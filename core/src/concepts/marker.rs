/// This is an implementation for Write + !AsRef<AnyBytes>.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsWritable;
unsafe impl Send for AsWritable {}
unsafe impl Sync for AsWritable {}

/// This is an implementation for Vec<u8>.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsOwnedBytes;
unsafe impl Send for AsOwnedBytes {}
unsafe impl Sync for AsOwnedBytes {}

/// This is an implementation for String.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsOwnedString;
unsafe impl Send for AsOwnedString {}
unsafe impl Sync for AsOwnedString {}
