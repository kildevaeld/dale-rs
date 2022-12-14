use crate::{into_outcome::IntoOutcome, outcome::Outcome, service::Service};
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use either::Either;
use futures_core::ready;
use pin_project_lite::pin_project;

pub struct Or<T1, T2, I> {
    left: T1,
    right: T2,
    _i: PhantomData<I>,
}

impl<T1: Clone, T2: Clone, I> Clone for Or<T1, T2, I> {
    fn clone(&self) -> Self {
        Or {
            left: self.left.clone(),
            right: self.right.clone(),
            _i: PhantomData,
        }
    }
}

impl<T1: Copy, T2: Copy, I> Copy for Or<T1, T2, I> {}

impl<T1, T2, I> Or<T1, T2, I> {
    pub fn new(left: T1, right: T2) -> Or<T1, T2, I> {
        Or {
            left,
            right,
            _i: PhantomData,
        }
    }
}

impl<T1, T2, R> Service<R> for Or<T1, T2, R>
where
    T1: Service<R>,
    T1::Future: 'static,
    T2: Service<R> + Clone + 'static,
{
    type Output = Outcome<
        Either<<T1::Output as IntoOutcome<R>>::Success, <T2::Output as IntoOutcome<R>>::Success>,
        Either<<T1::Output as IntoOutcome<R>>::Failure, <T2::Output as IntoOutcome<R>>::Failure>,
        R,
    >;

    type Future = OrElseFuture<T1, T2, R>;

    fn call(&self, req: R) -> Self::Future {
        OrElseFuture {
            state: OrElseFutureState::First {
                future: self.left.call(req),
                task: self.right.clone(),
            },
        }
    }
}

pin_project! {

    #[project = OrProj]
    enum OrElseFutureState<T1: Service<R>, T2: Service<R>, R>
    {
        First {
            #[pin]
            future: T1::Future,
            task:T2
        },
        Second {
            #[pin]
            future: T2::Future
        },
        Done,
    }

}

pin_project! {

    pub struct OrElseFuture<T1, T2, R>
    where
        T1: Service<R>,
        T2: Service<R>,
    {
        #[pin]
        state: OrElseFutureState<T1, T2, R>,
    }

}

impl<T1, T2, R> Future for OrElseFuture<T1, T2, R>
where
    T1: Service<R>,
    T2: Service<R>,
{
    #[allow(clippy::type_complexity)]

    type Output = Outcome<
        Either<<T1::Output as IntoOutcome<R>>::Success, <T2::Output as IntoOutcome<R>>::Success>,
        Either<<T1::Output as IntoOutcome<R>>::Failure, <T2::Output as IntoOutcome<R>>::Failure>,
        R,
    >;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let pin = self.as_mut().project();
            let fut2 = match pin.state.project() {
                OrProj::First {
                    future: first,
                    task: second,
                } => match ready!(first.poll(cx)).into_outcome() {
                    Outcome::Success(ret) => {
                        self.set(OrElseFuture {
                            state: OrElseFutureState::Done,
                        });
                        return Poll::Ready(Outcome::Success(Either::Left(ret)));
                    }
                    Outcome::Failure(err) => {
                        self.set(OrElseFuture {
                            state: OrElseFutureState::Done,
                        });
                        return Poll::Ready(Outcome::Failure(Either::Left(err)));
                    }
                    Outcome::Next(next) => second.call(next),
                },
                OrProj::Second { future } => {
                    let ret = match ready!(future.poll(cx)).into_outcome() {
                        Outcome::Success(ret) => Outcome::Success(Either::Right(ret)),
                        Outcome::Failure(err) => Outcome::Failure(Either::Right(err)),
                        Outcome::Next(next) => Outcome::Next(next),
                    };
                    self.set(OrElseFuture {
                        state: OrElseFutureState::Done,
                    });
                    return Poll::Ready(ret);
                }
                OrProj::Done => panic!("poll after done"),
            };

            self.set(OrElseFuture {
                state: OrElseFutureState::Second { future: fut2 },
            });
        }
    }
}
