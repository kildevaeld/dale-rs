use bytes::Buf;
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

    fn limit(self, limit: usize) -> LimitBody<Self> {
        LimitBody {
            inner: self,
            limit,
            read: 0,
        }
    }
}

pub enum LimitError<E> {
    Body(E),
    PayloadTooLarge,
}

impl<B> BodyExt for B where B: Body {}

pin_project_lite::pin_project! {
    pub struct LimitBody<B> {
        #[pin]
        inner: B,
        limit: usize,
        read: usize,
    }

}

impl<B> Body for LimitBody<B>
where
    B: Body,
{
    fn empty() -> Self {
        LimitBody {
            inner: B::empty(),
            limit: 0,
            read: 0,
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        LimitBody {
            limit: bytes.len(),
            inner: B::from_bytes(bytes),
            read: 0,
        }
    }

    #[cfg(feature = "stream")]
    fn from_stream<S, O, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<O, E>> + Send + 'static,
        O: Into<Bytes> + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    {
        LimitBody {
            limit: 0,
            inner: B::from_stream(stream),
            read: 0,
        }
    }
}

impl<B> HttpBody for LimitBody<B>
where
    B: HttpBody,
{
    type Data = B::Data;

    type Error = LimitError<B::Error>;

    fn poll_data(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();

        if this.read > this.limit {
            return core::task::Poll::Ready(Some(Err(LimitError::PayloadTooLarge)));
        }

        match futures_core::ready!(this.inner.poll_data(cx)) {
            Some(Ok(data)) => {
                let len = data.remaining();
                *this.read += len;
                core::task::Poll::Ready(Some(Ok(data)))
            }
            Some(Err(err)) => core::task::Poll::Ready(Some(Err(LimitError::Body(err)))),
            None => core::task::Poll::Ready(None),
        }
    }

    fn poll_trailers(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<Option<http::header::HeaderMap>, Self::Error>> {
        let this = self.project();
        match futures_core::ready!(this.inner.poll_trailers(cx)) {
            Ok(ret) => core::task::Poll::Ready(Ok(ret)),
            Err(err) => core::task::Poll::Ready(Err(LimitError::Body(err))),
        }
    }
}
