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

//

impl<'a, T1, T2, T3, B: 'static> FromRequest<'a, B> for (T1, T2, T3)
where
    T1: FromRequest<'a, B>,
    T2: FromRequest<'a, B>,
    T3: FromRequest<'a, B>,
{
    type Future = FromRequestT3Future<'a, T1, T2, T3, B>;

    fn from_request(request: &'a mut Request<B>) -> Self::Future {
        let ptr = request as *mut _;

        FromRequestT3Future::T1 {
            future: T1::from_request(request),
            right: PhantomData,
            req: ptr,
        }
    }
}

pin_project! {
    #[project = FromRequestT3FutureProj]
    pub enum FromRequestT3Future<'a, T1, T2, T3, B: 'static> where T1: FromRequest<'a, B>, T2: FromRequest<'a, B>, T3: FromRequest<'a, B> {
        T1 {
            #[pin]
            future: T1::Future,
            right: PhantomData<(T2, T3)>,
            req: *mut Request<B>,
        },
        T2 {
            #[pin]
            future: T2::Future,
            right_result: Option<T1>,
            req: *mut Request<B>,

        },
        T3 {
            #[pin]
            future: T3::Future,
            right_result: Option<(T1, T2)>
        },
        Done
    }
}

impl<'a, T1, T2, T3, B: 'static> Future for FromRequestT3Future<'a, T1, T2, T3, B>
where
    T1: FromRequest<'a, B>,
    T2: FromRequest<'a, B>,
    T3: FromRequest<'a, B>,
{
    type Output = Result<(T1, T2, T3), Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();
            let pro = match this {
                FromRequestT3FutureProj::T1 { future, req, .. } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => {
                            self.set(FromRequestT3Future::Done);
                            return Poll::Ready(Err(err));
                        }
                    };

                    FromRequestT3Future::T2 {
                        future: T2::from_request(unsafe { &mut **req }),
                        req: *req,
                        right_result: ret.into(),
                    }
                }
                FromRequestT3FutureProj::T2 {
                    future,
                    req,
                    right_result,
                } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    FromRequestT3Future::T3 {
                        future: T3::from_request(unsafe { &mut **req }),
                        right_result: Some((right_result.take().unwrap(), ret)),
                    }
                }
                FromRequestT3FutureProj::T3 {
                    future,
                    right_result,
                } => {
                    let (t1, t2) = right_result.take().unwrap();
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => Ok((t1, t2, ret)),
                        Err(err) => Err(err),
                    };

                    self.set(Self::Done);

                    return Poll::Ready(ret);
                }
                FromRequestT3FutureProj::Done => {
                    panic!("poll after done")
                }
            };

            self.set(pro);
        }
    }
}

//

impl<'a, T1, T2, T3, T4, B: 'static> FromRequest<'a, B> for (T1, T2, T3, T4)
where
    T1: FromRequest<'a, B>,
    T2: FromRequest<'a, B>,
    T3: FromRequest<'a, B>,
    T4: FromRequest<'a, B>,
{
    type Future = FromRequestT4Future<'a, T1, T2, T3, T4, B>;

    fn from_request(request: &'a mut Request<B>) -> Self::Future {
        let ptr = request as *mut _;

        FromRequestT4Future::T1 {
            future: T1::from_request(request),
            req: ptr,
        }
    }
}

pin_project! {
    #[project = FromRequestT4FutureProj]
    pub enum FromRequestT4Future<'a, T1, T2, T3, T4, B: 'static> where T1: FromRequest<'a, B>, T2: FromRequest<'a, B>, T3: FromRequest<'a, B> , T4: FromRequest<'a, B>{
        T1 {
            #[pin]
            future: T1::Future,
            req: *mut Request<B>,
        },
        T2 {
            #[pin]
            future: T2::Future,
            right_result: Option<T1>,
            req: *mut Request<B>,

        },
        T3 {
            #[pin]
            future: T3::Future,
            right_result: Option<(T1, T2)>,
            req: *mut Request<B>,

        },
        T4 {
            #[pin]
            future: T4::Future,
            right_result: Option<(T1, T2, T3)>
        },
        Done
    }
}

impl<'a, T1, T2, T3, T4, B: 'static> Future for FromRequestT4Future<'a, T1, T2, T3, T4, B>
where
    T1: FromRequest<'a, B>,
    T2: FromRequest<'a, B>,
    T3: FromRequest<'a, B>,
    T4: FromRequest<'a, B>,
{
    type Output = Result<(T1, T2, T3, T4), Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();
            let pro = match this {
                FromRequestT4FutureProj::T1 { future, req, .. } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => {
                            self.set(FromRequestT4Future::Done);
                            return Poll::Ready(Err(err));
                        }
                    };

                    FromRequestT4Future::T2 {
                        future: T2::from_request(unsafe { &mut **req }),
                        req: *req,
                        right_result: ret.into(),
                    }
                }
                FromRequestT4FutureProj::T2 {
                    future,
                    req,
                    right_result,
                } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    FromRequestT4Future::T3 {
                        future: T3::from_request(unsafe { &mut **req }),
                        req: *req,
                        right_result: Some((right_result.take().unwrap(), ret)),
                    }
                }
                FromRequestT4FutureProj::T3 {
                    future,
                    req,
                    right_result,
                } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    FromRequestT4Future::T4 {
                        future: T4::from_request(unsafe { &mut **req }),

                        right_result: right_result.take().map(|(t1, t2)| (t1, t2, ret)),
                    }
                }
                FromRequestT4FutureProj::T4 {
                    future,
                    right_result,
                } => {
                    let (t1, t2, t3) = right_result.take().unwrap();
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => Ok((t1, t2, t3, ret)),
                        Err(err) => Err(err),
                    };

                    self.set(Self::Done);

                    return Poll::Ready(ret);
                }
                FromRequestT4FutureProj::Done => {
                    panic!("poll after done")
                }
            };

            self.set(pro);
        }
    }
}
