use std::borrow::{Borrow, Cow};

use serde::{Deserializer, Serializer};
use serde_with::{
    base64::{Base64, Standard},
    formats::Padded,
};
use serde_with::{DeserializeAs, SerializeAs};

/// This is an implementation for `Write + !AsRef<AnyBytes>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsWritable;
unsafe impl Send for AsWritable {}
unsafe impl Sync for AsWritable {}

/// This is an implementation for `Vec<u8>`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsOwnedBytes;
unsafe impl Send for AsOwnedBytes {}
unsafe impl Sync for AsOwnedBytes {}

/// This is an implementation for `String`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsOwnedString;
unsafe impl Send for AsOwnedString {}
unsafe impl Sync for AsOwnedString {}

pub struct AsCowBytes;

type StdBase64 = Base64<Standard, Padded>;

impl<'b> SerializeAs<Cow<'b, [u8]>> for AsCowBytes {
    fn serialize_as<S>(source: &Cow<'b, [u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let t: &[u8] = source.borrow();
        StdBase64::serialize_as(&t, serializer)
    }
}

impl<'b, 'de> DeserializeAs<'de, Cow<'b, [u8]>> for AsCowBytes {
    fn deserialize_as<D>(deserializer: D) -> Result<Cow<'b, [u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf: Vec<u8> = StdBase64::deserialize_as(deserializer)?;
        Ok(Cow::Owned(buf))
    }
}
