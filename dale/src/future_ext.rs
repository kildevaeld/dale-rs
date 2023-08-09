use core::{marker::PhantomData, task::Poll};
use futures_core::{ready, Future};
use pin_project_lite::pin_project;

use crate::{IntoOutcome, Outcome};

pub trait DaleFutureExt: Future {
    fn into_outcome<N>(self) -> OutcomeFuture<Self, N>
    where
        Self: Sized,
        Self::Output: IntoOutcome<N>,
    {
        OutcomeFuture {
            future: self,
            _next: PhantomData,
        }
    }
}

impl<F> DaleFutureExt for F where F: Future {}

pin_project! {
    pub struct OutcomeFuture<F, N> {
        #[pin]
        future: F,
        _next: PhantomData<N>,
    }

}

unsafe impl<F: Send, N> Send for OutcomeFuture<F, N> {}

impl<F, N> Future for OutcomeFuture<F, N>
where
    F: Future,
    F::Output: IntoOutcome<N>,
{
    type Output =
        Outcome<<F::Output as IntoOutcome<N>>::Success, <F::Output as IntoOutcome<N>>::Failure, N>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        Poll::Ready(ready!(self.project().future.poll(cx)).into_outcome())
    }
}
