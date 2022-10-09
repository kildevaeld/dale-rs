use crate::{Either, Outcome, Service};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

#[derive(Clone, Copy, Debug)]
pub struct Unify<F> {
    pub(super) filter: F,
}

impl<F> Unify<F> {
    pub const fn new(filter: F) -> Unify<F> {
        Unify { filter }
    }
}

impl<F, T, E, R> Service<R> for Unify<F>
where
    F: Service<R, Output = Outcome<Either<T, T>, Either<E, E>, R>>,
{
    type Output = Outcome<T, E, R>;

    type Future = UnifyFuture<F, R, E>;
    #[inline]
    fn call(&self, req: R) -> Self::Future {
        UnifyFuture {
            inner: self.filter.call(req),
            _r: PhantomData,
            _e: PhantomData,
        }
    }
}

pin_project! {
    pub struct UnifyFuture<F, R, E> where F: Service<R> {
        #[pin]
        inner: F::Future,
        _r: PhantomData<R>,
        _e: PhantomData<E>,
    }

}

impl<F, R, E, T> Future for UnifyFuture<F, R, E>
where
    F: Service<R, Output = Outcome<Either<T, T>, Either<E, E>, R>>,
{
    type Output = Outcome<T, E, R>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let unified = match ready!(self.project().inner.poll(cx)) {
            Outcome::Next(next) => Outcome::Next(next),
            Outcome::Success(ret) => Outcome::Success(ret.into_inner()),
            Outcome::Failure(err) => Outcome::Failure(err.into_inner()),
        };
        Poll::Ready(unified)
    }
}
