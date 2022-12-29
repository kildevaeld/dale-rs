use super::error::DecodeError;
use crate::{common::Aggregate, Body};
use bytes::Buf;
use futures_core::{ready, Future};
use http::Request;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

pub trait Decodable {}

pub trait Decoder {
    type Error: Into<DecodeError>;
    const MIME: (mime::Name<'static>, mime::Name<'static>);
    const WITH_NO_CONTENT_TYPE: bool;

    fn decode<B: Buf, T: DeserializeOwned>(buf: B) -> Result<T, Self::Error>;

    fn check_content_type<B>(req: &Request<B>) -> Result<(), DecodeError> {
        let (type_, subtype) = Self::MIME;
        if let Some(value) = req.headers().get(http::header::CONTENT_TYPE) {
            tracing::trace!("is_content_type {}/{}? {:?}", type_, subtype, value);
            let ct = value
                .to_str()
                .ok()
                .and_then(|s| s.parse::<mime::Mime>().ok());
            if let Some(ct) = ct {
                if ct.type_() == type_ && ct.subtype() == subtype {
                    Ok(())
                } else {
                    tracing::debug!(
                        "content-type {:?} doesn't match {}/{}",
                        value,
                        type_,
                        subtype
                    );
                    Err(DecodeError::unsupported())
                }
            } else {
                tracing::debug!("content-type {:?} couldn't be parsed", value);
                Err(DecodeError::unsupported())
            }
        } else if Self::WITH_NO_CONTENT_TYPE {
            // Optimistically assume its correct!
            tracing::trace!("no content-type header, assuming {}/{}", type_, subtype);
            Ok(())
        } else {
            tracing::debug!("no content-type found");
            Err(DecodeError::unsupported())
        }
    }
}

pin_project! {

    pub struct ToDecoded<D, S, B>
    where
        B: Body,
    {
        #[pin]
        body: Aggregate<B>,
        _s: PhantomData<(S, D)>,
    }

}

impl<D, S, B> ToDecoded<D, S, B>
where
    B: Body,
{
    pub fn new(body: B) -> ToDecoded<D, S, B> {
        ToDecoded {
            body: Aggregate::new(body),
            _s: PhantomData,
        }
    }
}

unsafe impl<D, S, B: Body + Send> Send for ToDecoded<D, S, B> {}

impl<D, S, B> Future for ToDecoded<D, S, B>
where
    B: Body,
    B::Error: std::error::Error + Send + Sync + 'static,
    D: Decoder,
    S: DeserializeOwned,
{
    type Output = Result<S, DecodeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match ready!(this.body.poll(cx)) {
            Ok(ret) => match D::decode(ret) {
                Ok(ret) => Poll::Ready(Ok(ret)),
                Err(err) => Poll::Ready(Err(err.into())),
            },
            Err(err) => Poll::Ready(Err(DecodeError::transport(err))),
        }
    }
}
