use bytes::Buf;
use serde::de::DeserializeOwned;

pub trait Decoder {
    type Error;
    const MIME: (mime::Name<'static>, mime::Name<'static>);
    const WITH_NO_CONTENT_TYPE: bool;

    fn decode<B: Buf, T: DeserializeOwned>(buf: B) -> Result<T, Self::Error>;
}
