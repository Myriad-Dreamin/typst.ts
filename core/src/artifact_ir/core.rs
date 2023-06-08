pub use ecow::EcoString as TypstEcoString;
use serde::Deserialize;
use serde::Serialize;
pub use typst::doc::Destination as TypstDestination;
pub use typst::doc::FrameItem as TypstFrameItem;
pub use typst::doc::GroupItem as TypstGroupItem;
pub use typst::doc::Position as TypstPosition;
pub use typst::doc::TextItem as TypstTextItem;
pub use typst::geom::Shape as TypstShape;
pub use typst::model::Location as TypstLocation;
pub use typst::syntax::Span as TypstSpan;

use super::alloc::item_align_up;
use super::geom::Abs;

pub type SpanRef = ();
pub type GlyphShapeRef = u32;
pub type FontRef = u32;
pub type PaintRef = i32;
pub type Lang = String;
pub type EcoString = String;

/// Stably identifies an element in the document across multiple layout passes.
///
/// This struct is created by [`StabilityProvider::locate`].
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    /// The hash of the element.
    pub hash: u128,
    /// An unique number among elements with the same hash. This is the reason
    /// we need a mutable `StabilityProvider` everywhere.
    pub disambiguator: usize,
    /// A synthetic location created from another one. This is used for example
    /// in bibliography management to create individual linkable locations for
    /// reference entries from the bibliography's location.
    pub variant: usize,
}

#[repr(u32)]
#[derive(Clone, Debug, PartialOrd, Eq, Ord, Serialize, Deserialize, PartialEq)]
pub enum ItemRefKind {
    ItemWithPos,
    FrameItem,
    Frame,
    Glyph,
    PathItem,
    // Bytes,
    Abs,
    // String, // null-terminated
    MAX,
}

pub trait HasItemRefKind {
    const ITEM_REF_KIND: ItemRefKind;
}

impl HasItemRefKind for Abs {
    const ITEM_REF_KIND: ItemRefKind = ItemRefKind::Abs;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemRef<T> {
    pub offset: u32,

    #[serde(skip)]
    pub phantom: std::marker::PhantomData<T>,
}

impl<T: HasItemRefKind> ItemRef<T> {
    pub fn deref(&self, buffer: &[u8]) -> &T {
        let off = self.offset as usize;
        if off + std::mem::size_of::<T>() > buffer.len() {
            panic!(
                "buffer overflow in ItemRef::deref: off={}, size={}",
                off,
                std::mem::size_of::<T>()
            );
        }
        unsafe { &*(buffer.as_ptr().add(off) as *const T) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct StringRef {
    pub id: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct BytesRef {
    pub id: u32,
}

// if T::ITEM_REF_KIND == ItemRefKind::String {
//     panic!()
// }
// if T::ITEM_REF_KIND == ItemRefKind::String || T::ITEM_REF_KIND == ItemRefKind::Bytes {
//     panic!()
// }
// impl StringRef {
//     pub fn as_str(&self, buffer: &[u8]) -> &str {
//         let off = self.offset as usize;
//         unsafe {
//             let begin = buffer.as_ptr().add(off) as *const u8;
//             let mut end = begin;
//             // strlen for begin
//             while end.read() != 0 {
//                 if *end == 0 {
//                     break;
//                 }
//                 end = end.add(1);
//             }
//             std::str::from_utf8(std::slice::from_raw_parts(
//                 begin,
//                 end.offset_from(begin) as usize,
//             ))
//             .unwrap()
//         }
//     }
// }

// impl BytesRef {
//     pub fn as_slice(&self, buffer: &[u8], len: usize) -> &[u8] {
//         let off = self.offset as usize;
//         unsafe {
//             let begin = buffer.as_ptr().add(off) as *const u8;
//             std::slice::from_raw_parts(begin, len)
//         }
//     }
// }

/// Represents a series of items with contiguous ids.
#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemArray<T> {
    pub start: u32,
    pub size: u32,

    #[serde(skip)]
    pub phantom: std::marker::PhantomData<T>,
}

impl<'a, T: HasItemRefKind> ItemArray<T> {
    pub fn iter(&'a self, buffer: &'a [u8]) -> ItemArrayIter<'a, T> {
        let max_visit =
            self.start as usize + item_align_up(std::mem::size_of::<T>()) * self.size as usize;
        if max_visit > buffer.len() {
            panic!(
                "buffer overflow in ItemRef::deref: off={}, size={}",
                max_visit,
                buffer.len(),
            );
        }
        ItemArrayIter {
            array: self,
            index: 0,
            buffer,
        }
    }
}

impl<T: HasItemRefKind + Clone> ItemArray<T> {
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn len(&self) -> usize {
        self.size as usize
    }

    pub fn to_vec(&self, buffer: &[u8]) -> Vec<T> {
        let mut vec = Vec::with_capacity(self.size as usize);
        for x in self.iter(buffer) {
            vec.push((*x).clone());
        }
        vec
    }
}

impl<T> Default for ItemArray<T> {
    fn default() -> Self {
        Self {
            start: 0,
            size: 0,
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct ItemArrayIter<'a, T> {
    array: &'a ItemArray<T>,
    index: u32,
    buffer: &'a [u8],
}

impl<'a, T> Iterator for ItemArrayIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.size {
            let off = self.array.start as usize
                + self.index as usize * item_align_up(std::mem::size_of::<T>());
            let item = unsafe { &*(self.buffer.as_ptr().add(off) as *const T) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T> ExactSizeIterator for ItemArrayIter<'_, T> {
    fn len(&self) -> usize {
        self.array.size as usize
    }
}
