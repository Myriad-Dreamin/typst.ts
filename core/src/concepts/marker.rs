/// This is an implementation for Write + !AsRef<AnyBytes>.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsWritable;
unsafe impl Send for AsWritable {}
unsafe impl Sync for AsWritable {}
