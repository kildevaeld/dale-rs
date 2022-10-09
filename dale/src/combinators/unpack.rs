use crate::filters::Extract;
use crate::{IntoOutcome, Outcome, Service};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

#[derive(Clone, Copy, Debug)]
pub struct Unpack<S> {
    pub(super) filter: S,
}

impl<S> Unpack<S> {
    pub const fn new(filter: S) -> Unpack<S> {
        Unpack { filter }
    }
}

impl<S, R> Service<R> for Unpack<S>
where
    S: Service<R>,
    <S::Output as IntoOutcome<R>>::Success: Extract<R>,
{
    type Output = Outcome<
        <<S::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract,
        <S::Output as IntoOutcome<R>>::Failure,
        R,
    >;

    type Future = UnpackFuture<S, R>;
    #[inline]
    fn call(&self, req: R) -> Self::Future {
        UnpackFuture {
            inner: self.filter.call(req),
            _r: PhantomData,
        }
    }
}

pin_project! {
    pub struct UnpackFuture<S, R> where S: Service<R> {
        #[pin]
        inner: S::Future,
        _r: PhantomData<R>,
    }

}

impl<S, R> Future for UnpackFuture<S, R>
where
    S: Service<R>,
    <S::Output as IntoOutcome<R>>::Success: Extract<R>,
{
    type Output = Outcome<
        <<S::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract,
        <S::Output as IntoOutcome<R>>::Failure,
        R,
    >;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let unified = match ready!(self.project().inner.poll(cx)).into_outcome() {
            Outcome::Next(next) => Outcome::Next(next),
            Outcome::Success(ret) => {
                let (_, all) = ret.unpack();
                Outcome::Success(all)
            }
            Outcome::Failure(err) => Outcome::Failure(err),
        };
        Poll::Ready(unified)
    }
}
