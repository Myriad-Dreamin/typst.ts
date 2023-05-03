use serde::Deserialize;
use serde::Serialize;
pub use typst::syntax::Span as TypstSpan;
pub use typst_library::prelude::Destination as TypstDestination;
pub use typst_library::prelude::EcoString as TypstEcoString;
pub use typst_library::prelude::FrameItem as TypstFrameItem;
pub use typst_library::prelude::GroupItem as TypstGroupItem;
pub use typst_library::prelude::Location as TypstLocation;
pub use typst_library::prelude::Position as TypstPosition;
pub use typst_library::prelude::Shape as TypstShape;
pub use typst_library::prelude::TextItem as TypstTextItem;

pub type SpanRef = ();
pub type FontRef = u32;
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

#[repr(C)]
#[derive(Clone, Debug, PartialOrd, Eq, Ord, Serialize, Deserialize, PartialEq)]
pub enum ItemRefKind {
    ItemWithPos,
    FrameItem,
    Frame,
    Glyph,
    Byte,
    String, // null-terminated
}

pub trait HasItemRefKind {
    const ITEM_REF_KIND: ItemRefKind;
}

impl HasItemRefKind for u8 {
    const ITEM_REF_KIND: ItemRefKind = ItemRefKind::Byte;
}

#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ItemRef<T> {
    pub id: u32,
    pub kind: ItemRefKind,

    #[serde(skip)]
    pub phantom: std::marker::PhantomData<T>,
}

impl<T: HasItemRefKind> ItemRef<T> {
    pub fn deref(&self, buffer: &[u8]) -> &T {
        if T::ITEM_REF_KIND == ItemRefKind::String {
            panic!()
        }
        let off = self.id as usize;
        unsafe { &*(buffer.as_ptr().add(off) as *const T) }
    }
}

impl ItemRef<String> {
    pub fn as_str(&self, buffer: &[u8]) -> &str {
        let off = self.id as usize;
        unsafe {
            let begin = buffer.as_ptr().add(off) as *const u8;
            let mut end = begin;
            // strlen for begin
            while end.read() != 0 {
                if *end == 0 {
                    break;
                }
                end = end.add(1);
            }
            std::str::from_utf8(std::slice::from_raw_parts(
                begin,
                end.offset_from(begin) as usize,
            ))
            .unwrap()
        }
    }
}

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
        if T::ITEM_REF_KIND == ItemRefKind::String {
            panic!()
        }
        ItemArrayIter {
            array: self,
            index: 0,
            buffer,
        }
    }
}

impl<T: HasItemRefKind + Clone> ItemArray<T> {
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
            let off = self.array.start as usize + self.index as usize * std::mem::size_of::<T>();
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
