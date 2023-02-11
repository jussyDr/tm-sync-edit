use base64::prelude::{Engine, BASE64_STANDARD};
use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Base64<B>(B);

impl<B> From<B> for Base64<B> {
    fn from(bytes: B) -> Self {
        Self(bytes)
    }
}

impl<B> Serialize for Base64<B>
where
    B: AsRef<[u8]>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let base64 = BASE64_STANDARD.encode(&self.0);
        serializer.serialize_str(&base64)
    }
}

impl<'de, B> Deserialize<'de> for Base64<B>
where
    B: TryFrom<Vec<u8>>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Base64Visitor<B> {
            marker: PhantomData<Base64<B>>,
        }

        impl<B> Visitor<'_> for Base64Visitor<B>
        where
            B: TryFrom<Vec<u8>>,
        {
            type Value = Base64<B>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let bytes = BASE64_STANDARD
                    .decode(s)
                    .map_err(|_| E::invalid_value(Unexpected::Str(s), &"a base64 string"))?;

                let bytes = bytes
                    .try_into()
                    .map_err(|_| E::custom("conversion failed"))?;

                Ok(Base64(bytes))
            }
        }

        let visitor = Base64Visitor {
            marker: PhantomData,
        };

        deserializer.deserialize_str(visitor)
    }
}
