use core::{
    pin::Pin,
    task::{Context, Poll},
};

use either::Either;
use futures_core::{ready, Future};
use pin_project_lite::pin_project;

use crate::{types::MapFunc, IntoOutcome, Outcome, Service};

pub struct RequireService<T, F> {
    service: T,
    func: F,
}

impl<T, F> RequireService<T, F> {
    pub fn new(service: T, func: F) -> RequireService<T, F> {
        RequireService { service, func }
    }
}

impl<T, F, R> Service<R> for RequireService<T, F>
where
    T: Service<R>,
    F: MapFunc<R> + Clone,
{
    type Output = Outcome<
        <T::Output as IntoOutcome<R>>::Success,
        Either<<T::Output as IntoOutcome<R>>::Failure, F::Output>,
        R,
    >;

    type Future = RequireServiceFuture<T, F, R>;

    fn call(&self, req: R) -> Self::Future {
        RequireServiceFuture {
            state: State::Init {
                future: self.service.call(req),
                func: self.func.clone(),
            },
        }
    }
}

pin_project! {

    pub struct RequireServiceFuture<T, F, R>
    where
        T: Service<R>,
        F: MapFunc<R>,
    {
        #[pin]
        state: State<T, F, R>
    }
}

pin_project! {
    #[project = StateProject]
    enum State<T, F, R>
    where
    T: Service<R>,
    F: MapFunc<R>,
    {
        Init {
            #[pin]
            future: T::Future,
            func: F
        },
        Done,
    }
}

impl<T, F, R> Future for RequireServiceFuture<T, F, R>
where
    T: Service<R>,
    F: MapFunc<R>,
{
    type Output = Outcome<
        <T::Output as IntoOutcome<R>>::Success,
        Either<<T::Output as IntoOutcome<R>>::Failure, F::Output>,
        R,
    >;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match this.state.project() {
            StateProject::Init { future, func } => {
                let ret = match ready!(future.poll(cx)).into_outcome() {
                    Outcome::Success(ret) => Outcome::Success(ret),
                    Outcome::Failure(err) => Outcome::Failure(Either::Left(err)),
                    Outcome::Next(next) => {
                        let mapped_next = func.call(next);
                        Outcome::Failure(Either::Right(mapped_next))
                    }
                };

                self.set(RequireServiceFuture { state: State::Done });

                Poll::Ready(ret)
            }
            StateProject::Done => {
                panic!("poll after done")
            }
        }
    }
}
