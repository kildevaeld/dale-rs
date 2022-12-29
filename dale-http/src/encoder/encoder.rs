use bytes::Bytes;
use serde::Serialize;

use super::EncodeError;

pub trait Encoder {
    type Error: Into<EncodeError>;
    const MIME: mime::Mime;
    fn encode<S: Serialize>(data: &S) -> Result<Bytes, Self::Error>;
    fn encode_pretty<S: Serialize>(data: &S) -> Result<Bytes, Self::Error> {
        Self::encode(data)
    }
}
