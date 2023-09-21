use std::{marker::PhantomData, task::Poll};

use futures_core::{ready, Future};
use http::Request;
use pin_project_lite::pin_project;

use crate::Error;

pub trait FromRequest<'a, B: 'static>: Sized {
    type Future: Future<Output = Result<Self, Error>>;

    fn from_request(request: &'a mut Request<B>) -> Self::Future;
}

impl<'a, L, R, B: 'static> FromRequest<'a, B> for (L, R)
where
    L: FromRequest<'a, B>,
    R: FromRequest<'a, B>,
{
    type Future = FromRequestFuture<'a, L, R, B>;

    fn from_request(request: &'a mut Request<B>) -> Self::Future {
        let ptr = request as *mut _;

        FromRequestFuture::Left {
            future: L::from_request(request),
            right: PhantomData,
            req: ptr,
        }
    }
}

pin_project! {
    #[project = EnumProj]
    pub enum FromRequestFuture<'a, L, R, B: 'static> where L: FromRequest<'a, B>, R: FromRequest<'a, B> {
        Left {
            #[pin]
            future: L::Future,
            right: PhantomData<R>,
            req: *mut Request<B>,
        },
        Right {
            #[pin]
            future: R::Future,
            right_result: Option<L>

        },
        Done
    }
}

impl<'a, L, R, B: 'static> Future for FromRequestFuture<'a, L, R, B>
where
    L: FromRequest<'a, B>,
    R: FromRequest<'a, B>,
{
    type Output = Result<(L, R), Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();
            let pro = match this {
                EnumProj::Left { future, req, .. } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => {
                            self.set(FromRequestFuture::Done);
                            return Poll::Ready(Err(err));
                        }
                    };

                    FromRequestFuture::Right {
                        future: R::from_request(unsafe { &mut **req }),
                        right_result: ret.into(),
                    }
                }
                EnumProj::Right {
                    future,
                    right_result,
                } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => Ok((right_result.take().unwrap(), ret)),
                        Err(err) => Err(err),
                    };

                    self.set(Self::Done);

                    return Poll::Ready(ret);
                }
                EnumProj::Done => {
                    panic!("poll after done")
                }
            };

            self.set(pro);
        }
    }
}
