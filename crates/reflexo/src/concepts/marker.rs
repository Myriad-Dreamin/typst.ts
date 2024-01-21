use std::borrow::{Borrow, Cow};

use serde::{Deserializer, Serializer};
use serde_with::{
    base64::{Base64, Standard},
    formats::Padded,
};
use serde_with::{DeserializeAs, SerializeAs};

/// Generic marker for type that only occurs in parameter position.
///
/// Safety: The signature is type checked by the compiler.
#[derive(Debug, Clone, Copy)]
pub struct PhantomParamData<X>(std::marker::PhantomData<X>);
unsafe impl<X> Send for PhantomParamData<X> {}

impl<X> Default for PhantomParamData<X> {
    fn default() -> Self {
        Self(Default::default())
    }
}

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
