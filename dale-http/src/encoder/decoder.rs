use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Buf;
use futures_core::{ready, Future};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;

use crate::{common::Aggregate, error::Error, Body};

pub trait Decodable {}

pub trait Decoder {
    type Error;
    const MIME: (mime::Name<'static>, mime::Name<'static>);
    const WITH_NO_CONTENT_TYPE: bool;

    fn decode<B: Buf, T: DeserializeOwned>(buf: B) -> Result<T, Self::Error>;
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
    B::Error: Into<Error>,
    D: Decoder,
    S: DeserializeOwned,
{
    type Output = Result<S, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match ready!(this.body.poll(cx)) {
            Ok(ret) => match D::decode(ret) {
                Ok(ret) => Poll::Ready(Ok(ret)),
                Err(_err) => {
                    // Poll::Ready(Err(err))
                    todo!()
                }
            },
            Err(err) => Poll::Ready(Err(err.into())),
        }
    }
}
