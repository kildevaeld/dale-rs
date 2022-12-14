use crate::{IntoOutcome, Outcome, Service};
use core::{future::Future, marker::PhantomData, pin::Pin, task::Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

pub struct MapErr<F, S, E> {
    func: F,
    service: S,
    _e: PhantomData<E>,
}

impl<F: Clone, S: Clone, E> Clone for MapErr<F, S, E> {
    fn clone(&self) -> Self {
        MapErr {
            func: self.func.clone(),
            service: self.service.clone(),
            _e: PhantomData,
        }
    }
}

impl<F: Copy, S: Copy, E> Copy for MapErr<F, S, E> {}

unsafe impl<F: Send, S: Send, E> Send for MapErr<F, S, E> {}

unsafe impl<F: Sync, S: Sync, E> Sync for MapErr<F, S, E> {}

impl<F, S, E> MapErr<F, S, E> {
    pub fn new(service: S, func: F) -> MapErr<F, S, E> {
        MapErr {
            func,
            service,
            _e: PhantomData,
        }
    }
}

impl<F, S, E, R> Service<R> for MapErr<F, S, E>
where
    S: Service<R>,
    F: Fn(<S::Output as IntoOutcome<R>>::Failure) -> E + Send + Clone,
    R: Send,
    E: Send,
{
    type Output = Outcome<<S::Output as IntoOutcome<R>>::Success, E, R>;

    type Future = MapErrFuture<F, S::Future, R, E>;

    fn call(&self, req: R) -> Self::Future {
        let fut = self.service.call(req);

        MapErrFuture {
            future: fut,
            func: self.func.clone(),
            _r: PhantomData,
            _e: PhantomData,
        }
    }
}

pin_project! {

    pub struct MapErrFuture<F, T, R, E> {
        #[pin]
        future: T,
        func: F,
        _r: PhantomData<R>,
        _e: PhantomData<E>
    }
}

impl<F, T, R, E> Future for MapErrFuture<F, T, R, E>
where
    T: Future,
    T::Output: IntoOutcome<R>,
    F: Fn(<T::Output as IntoOutcome<R>>::Failure) -> E,
{
    type Output = Outcome<<T::Output as IntoOutcome<R>>::Success, E, R>;

    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let ret = match ready!(this.future.poll(cx)).into_outcome() {
            Outcome::Failure(err) => Outcome::Failure((this.func)(err)),
            Outcome::Next(next) => Outcome::Next(next),

            Outcome::Success(success) => Outcome::Success(success),
        };

        Poll::Ready(ret)
    }
}
