use crate::{types::alloc::*, IntoOutcome, Outcome, Service};
use core::{future::Future, task::Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

pub struct VecService<S>(Arc<Vec<S>>);

impl<S> VecService<S> {
    pub fn new(services: Vec<S>) -> VecService<S> {
        VecService(Arc::new(services))
    }
}

impl<R, S> Service<R> for VecService<S>
where
    S: Service<R>,
{
    type Output =
        Outcome<<S::Output as IntoOutcome<R>>::Success, <S::Output as IntoOutcome<R>>::Failure, R>;

    type Future = VecServiceFuture<S, R>;

    fn call(&self, req: R) -> Self::Future {
        VecServiceFuture {
            state: State::Init {
                list: self.0.clone().into(),
                req: req.into(),
            },
        }
    }
}

pin_project! {

    pub struct VecServiceFuture<T, R> where T: Service<R> {
        #[pin]
        state: State<T, R>,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<T, R> where T: Service<R> {
        Init {
            list: Option<Arc<Vec<T>>>,
            req: Option<R>
        },
        Current {
            #[pin]
            future: T::Future,
            idx: usize,
            services: Option<Arc<Vec<T>>>
        },
        Done,
    }
}

impl<T, R> Future for VecServiceFuture<T, R>
where
    T: Service<R>,
{
    type Output =
        Outcome<<T::Output as IntoOutcome<R>>::Success, <T::Output as IntoOutcome<R>>::Failure, R>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let state = match this.state.project() {
                StateProj::Init { list, req } => {
                    let list = list.take().unwrap();
                    let req = req.take().unwrap();

                    if list.is_empty() {
                        self.set(VecServiceFuture { state: State::Done });
                        return Poll::Ready(Outcome::Next(req));
                    }

                    let future = list[0].call(req);

                    State::Current {
                        future,
                        idx: 0,
                        services: Some(list),
                    }
                }
                StateProj::Current {
                    future,
                    idx,
                    services,
                } => match ready!(future.poll(cx)).into_outcome() {
                    Outcome::Success(ret) => {
                        self.set(VecServiceFuture { state: State::Done });
                        return Poll::Ready(Outcome::Success(ret));
                    }
                    Outcome::Failure(err) => {
                        self.set(VecServiceFuture { state: State::Done });
                        return Poll::Ready(Outcome::Failure(err));
                    }
                    Outcome::Next(next) => {
                        let services = services.take().unwrap();
                        let idx = *idx + 1;
                        match services.get(idx) {
                            Some(ret) => {
                                let future = ret.call(next);
                                State::Current {
                                    future,
                                    idx,
                                    services: Some(services),
                                }
                            }
                            None => {
                                self.set(VecServiceFuture { state: State::Done });
                                return Poll::Ready(Outcome::Next(next));
                            }
                        }
                    }
                },
                StateProj::Done => {
                    panic!("poll after done")
                }
            };

            self.set(VecServiceFuture { state })
        }
    }
}
