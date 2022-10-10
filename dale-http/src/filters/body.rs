use crate::error::Error;
#[cfg(feature = "headers")]
use crate::error::KnownError;
use crate::{body::BodyExt, Body};
use bytes::{Buf, Bytes};
use dale::{Outcome, Service, ServiceExt};
use futures_core::Future;
#[cfg(feature = "headers")]
use headers::ContentLength;
use http::Request;

use std::{convert::Infallible, fmt};

macro_rules! service_impl {
    ($name: ident, $future: ident, $type: ident, fn ($req: ident) $func: block) => {
        pub struct $name;

        impl<B: Body> Service<Request<B>> for $name {
            type Output = Outcome<(Request<B>, ($type,)), Error, Request<B>>;
            type Future = $future<B>;

            fn call(&self, req: Request<B>) -> Self::Future {
                $future { req: Some(req) }
            }
        }

        pin_project_lite::pin_project! {
            pub struct $future<B> {
                req: Option<Request<B>>,
            }
        }

        impl<B: Body> std::future::Future for $future<B> {
            type Output = Outcome<(Request<B>, ($type,)), Error, Request<B>>;
            fn poll(
                self: core::pin::Pin<&mut Self>,
                _cx: &mut core::task::Context<'_>,
            ) -> core::task::Poll<Self::Output> {
                let this = self.project();

                let mut $req = this.req.take().expect("request");

                let ret = $func;

                core::task::Poll::Ready(ret)
            }
        }
    };
}

service_impl!(GetBody, GetBodyFuture, B, fn (req) {
    let body = std::mem::replace(req.body_mut(), B::empty());
    Outcome::Success((req, (body,)))
});

#[derive(Debug)]
pub struct BodyReadError<E>(pub E);

impl<E> fmt::Display for BodyReadError<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "body read error: {}", self.0)
    }
}

impl<E> std::error::Error for BodyReadError<E> where E: std::error::Error {}

#[cfg(feature = "headers")]
pub fn content_length_limit<B: Send + 'static>(
    limit: u64,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
> + Copy {
    crate::filters::header::header()
        .then(
            move |(req, (ContentLength(length),)): (Request<B>, (ContentLength,))| async move {
                if length <= limit {
                    Ok((req, ()))
                } else {
                    tracing::debug!("content-length: {} is over limit {}", length, limit);
                    Err(Error::from(KnownError::PayloadTooLarge))
                }
            },
        )
        .err_into()
}

pub fn body<B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (B,)), Infallible, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    |mut req: Request<B>| async move {
        let body = std::mem::replace(req.body_mut(), B::empty());
        Outcome::Success((req, (body,)))
    }
}

pub fn aggregate<B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (impl Buf,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    body().and_then(crate::common::Aggregate::new).err_into()
}

pub fn bytes<B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (Bytes,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    body().and_then(B::bytes).err_into()
}

pub fn text<B: Body + Send + 'static + Default>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (String,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    B::Error: std::error::Error + Send + Sync,
    B::Data: Send,
{
    body().and_then(B::text).err_into()
}
