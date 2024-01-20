use core::fmt;
use std::{
    borrow::Cow,
    ops::{Add, AddAssign, Deref},
    sync::Arc,
};

use comemo::Prehashed;
use serde::{Serialize, Serializer};

/// This type is borrow from typst.
///
/// A sequence of bytes.
///
/// This is conceptually similar to an array of [integers]($int) between `{0}`
/// and `{255}`, but represented much more efficiently.
///
/// You can convert
/// - a [string]($str) or an [array]($array) of integers to bytes with the
///   [`bytes`]($bytes) constructor
/// - bytes to a string with the [`str`]($str) constructor
/// - bytes to an array of integers with the [`array`]($array) constructor
///
/// When [reading]($read) data from a file, you can decide whether to load it
/// as a string or as raw bytes.
///
/// ```example
/// #bytes((123, 160, 22, 0)) \
/// #bytes("Hello 😃")
///
/// #let data = read(
///   "rhino.png",
///   encoding: none,
/// )
///
/// // Magic bytes.
/// #array(data.slice(0, 4)) \
/// #str(data.slice(1, 4))
/// ```
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Bytes(Arc<Prehashed<Cow<'static, [u8]>>>);

impl Bytes {
    /// Create a buffer from a static byte slice.
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self(Arc::new(Prehashed::new(Cow::Borrowed(slice))))
    }

    /// Return `true` if the length is 0.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return a view into the buffer.
    pub fn as_slice(&self) -> &[u8] {
        self
    }

    /// Return a copy of the buffer as a vector.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bytes({})", self.len())
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<&[u8]> for Bytes {
    fn from(slice: &[u8]) -> Self {
        Self(Arc::new(Prehashed::new(slice.to_vec().into())))
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Self(Arc::new(Prehashed::new(vec.into())))
    }
}

impl Add for Bytes {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Bytes {
    fn add_assign(&mut self, rhs: Self) {
        if rhs.is_empty() {
            // Nothing to do
        } else if self.is_empty() {
            *self = rhs;
        } else if Arc::strong_count(&self.0) == 1 && matches!(**self.0, Cow::Owned(_)) {
            Arc::make_mut(&mut self.0).update(|cow| {
                cow.to_mut().extend_from_slice(&rhs);
            })
        } else {
            *self = Self::from([self.as_slice(), rhs.as_slice()].concat());
        }
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("{self:?}"))
        } else {
            serializer.serialize_bytes(self)
        }
    }
}
