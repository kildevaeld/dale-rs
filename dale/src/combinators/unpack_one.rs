use crate::filters::ExtractOne;
use crate::{IntoOutcome, Outcome, Service};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

#[derive(Clone, Copy, Debug)]
pub struct UnpackOne<S> {
    pub(super) filter: S,
}

impl<S> UnpackOne<S> {
    pub const fn new(filter: S) -> UnpackOne<S> {
        UnpackOne { filter }
    }
}

impl<S, R> Service<R> for UnpackOne<S>
where
    S: Service<R>,
    <S::Output as IntoOutcome<R>>::Success: ExtractOne<R>,
{
    type Output = Outcome<
        <<S::Output as IntoOutcome<R>>::Success as ExtractOne<R>>::Output,
        <S::Output as IntoOutcome<R>>::Failure,
        R,
    >;

    type Future = UnpackOneFuture<S, R>;
    #[inline]
    fn call(&self, req: R) -> Self::Future {
        UnpackOneFuture {
            inner: self.filter.call(req),
            _r: PhantomData,
        }
    }
}

pin_project! {
    pub struct UnpackOneFuture<S, R> where S: Service<R> {
        #[pin]
        inner: S::Future,
        _r: PhantomData<R>,
    }

}

impl<S, R> Future for UnpackOneFuture<S, R>
where
    S: Service<R>,
    <S::Output as IntoOutcome<R>>::Success: ExtractOne<R>,
{
    type Output = Outcome<
        <<S::Output as IntoOutcome<R>>::Success as ExtractOne<R>>::Output,
        <S::Output as IntoOutcome<R>>::Failure,
        R,
    >;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let unified = match ready!(self.project().inner.poll(cx)).into_outcome() {
            Outcome::Next(next) => Outcome::Next(next),
            Outcome::Success(ret) => {
                let (_, one) = ret.unpack_one();
                Outcome::Success(one)
            }
            Outcome::Failure(err) => Outcome::Failure(err),
        };
        Poll::Ready(unified)
    }
}
