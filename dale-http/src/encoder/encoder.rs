use bytes::Bytes;
use serde::Serialize;

pub trait Encoder {
    type Error;
    const MIME: mime::Mime;
    fn encode<S: Serialize>(data: &S) -> Result<Bytes, Self::Error>;
    fn encode_pretty<S: Serialize>(data: &S) -> Result<Bytes, Self::Error> {
        Self::encode(data)
    }
}
