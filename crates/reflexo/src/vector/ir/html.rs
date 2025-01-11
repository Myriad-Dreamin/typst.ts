use super::preludes::*;

/// Item representing a sized html element in frames.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct SizedRawHtmlItem {
    /// The sanitized source code.
    pub html: ImmutStr,
    /// The target size of the image.
    pub size: Size,
}

/// Item representing a html element in frames.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct HtmlItem {
    /// The sanitized source code.
    pub html: ImmutStr,
}
