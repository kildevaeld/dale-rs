#[cfg(feature = "stream")]
use bytes::Bytes;
#[cfg(feature = "stream")]
use futures_core::Stream;
use http_body::Body as HttpBody;

use crate::common::{ToBytes, ToText};
#[cfg(feature = "json")]
use crate::encoder::Json;
#[cfg(feature = "serde")]
use crate::encoder::{Decoder, ToDecoded};
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;

pub trait Body: HttpBody + Sized {
    fn empty() -> Self;
    fn from_bytes(bytes: Vec<u8>) -> Self;

    #[cfg(feature = "stream")]
    fn from_stream<S, O, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<O, E>> + Send + 'static,
        O: Into<Bytes> + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static;
}

pub trait BodyExt: Body {
    fn bytes(self) -> ToBytes<Self> {
        ToBytes::new(self)
    }

    fn text(self) -> ToText<Self> {
        ToText::new(self)
    }

    #[cfg(feature = "serde")]
    fn decode<D: Decoder, S: DeserializeOwned>(self) -> ToDecoded<D, S, Self> {
        ToDecoded::new(self)
    }

    #[cfg(feature = "json")]
    fn json<S: DeserializeOwned>(self) -> ToDecoded<Json, S, Self> {
        self.decode()
    }
}

impl<B> BodyExt for B where B: Body {}
