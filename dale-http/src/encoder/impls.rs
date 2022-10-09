use super::{Decoder, Encoder};
use bytes::{Buf, Bytes};
use serde::{de::DeserializeOwned, Serialize};

pub struct Form;

impl Decoder for Form {
    type Error = serde_urlencoded::de::Error;
    const MIME: (mime::Name<'static>, mime::Name<'static>) =
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED);
    const WITH_NO_CONTENT_TYPE: bool = true;

    fn decode<B: Buf, T: DeserializeOwned>(buf: B) -> Result<T, Self::Error> {
        serde_urlencoded::from_reader(buf.reader())
    }
}

#[cfg(feature = "json")]
pub struct Json;

#[cfg(feature = "json")]
impl Encoder for Json {
    type Error = serde_json::Error;
    const MIME: mime::Mime = mime::APPLICATION_JSON;
    fn encode<S: Serialize>(data: &S) -> Result<Bytes, Self::Error> {
        Ok(Bytes::from(serde_json::to_vec(data)?))
    }

    fn encode_pretty<S: Serialize>(data: &S) -> Result<Bytes, Self::Error> {
        Ok(Bytes::from(serde_json::to_vec_pretty(data)?))
    }
}

#[cfg(feature = "json")]
impl Decoder for Json {
    type Error = serde_json::Error;
    const MIME: (mime::Name<'static>, mime::Name<'static>) = (mime::APPLICATION, mime::JSON);
    const WITH_NO_CONTENT_TYPE: bool = true;

    fn decode<B: Buf, T: DeserializeOwned>(buf: B) -> Result<T, Self::Error> {
        serde_json::from_reader(buf.reader())
    }
}
