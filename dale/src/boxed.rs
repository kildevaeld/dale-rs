use crate::{IntoOutcome, Middleware, Outcome, Service};
use core::{future::Future, pin::Pin};

#[cfg(not(feature = "std"))]
pub use alloc::boxed::Box;
#[cfg(feature = "std")]
pub use std::boxed::Box;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub type BoxService<'a, I, O, E> = Box<
    dyn Service<I, Output = Outcome<O, E, I>, Future = BoxFuture<'a, Outcome<O, E, I>>>
        + Send
        + Sync,
>;

pub type LocalBoxService<'a, I, O, E> =
    Box<dyn Service<I, Output = Outcome<O, E, I>, Future = LocalBoxFuture<'a, Outcome<O, E, I>>>>;

#[derive(Clone, Debug)]
pub(crate) struct BoxedService<S> {
    service: S,
}

impl<S> BoxedService<S> {
    pub fn new(service: S) -> BoxedService<S> {
        BoxedService { service }
    }
}

impl<S, I> Service<I> for BoxedService<S>
where
    S: Service<I>,
    S::Future: 'static + Send,
{
    type Output =
        Outcome<<S::Output as IntoOutcome<I>>::Success, <S::Output as IntoOutcome<I>>::Failure, I>;

    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, req: I) -> Self::Future {
        let future = self.service.call(req);
        Box::pin(async move { future.await.into_outcome() })
    }
}

impl<'a, I, O, E> Service<I> for BoxService<'a, I, O, E> {
    type Output = Outcome<O, E, I>;

    type Future = BoxFuture<'a, Outcome<O, E, I>>;

    fn call(&self, req: I) -> Self::Future {
        (**self).call(req)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LocalBoxedService<S> {
    service: S,
}

impl<S> LocalBoxedService<S> {
    pub fn new(service: S) -> LocalBoxedService<S> {
        LocalBoxedService { service }
    }
}

impl<S, I> Service<I> for LocalBoxedService<S>
where
    S: Service<I>,
    S::Future: 'static,
{
    type Output =
        Outcome<<S::Output as IntoOutcome<I>>::Success, <S::Output as IntoOutcome<I>>::Failure, I>;

    type Future = LocalBoxFuture<'static, Self::Output>;

    fn call(&self, req: I) -> Self::Future {
        let future = self.service.call(req);
        Box::pin(async move { future.await.into_outcome() })
    }
}

impl<'a, I, O, E> Service<I> for LocalBoxService<'a, I, O, E> {
    type Output = Outcome<O, E, I>;

    type Future = LocalBoxFuture<'a, Outcome<O, E, I>>;

    fn call(&self, req: I) -> Self::Future {
        (**self).call(req)
    }
}

pub type BoxMiddleware<'a, I, O, E, S> =
    Box<dyn Middleware<I, S, Service = BoxService<'a, I, O, E>> + Send + Sync>;

pub struct BoxedMiddleware<M>(pub M);

impl<'a, I, S, M> Middleware<I, S> for BoxedMiddleware<M>
where
    M: Middleware<I, S>,
    S: Service<I>,
    M::Service: Sync + Send + 'static,
    <M::Service as Service<I>>::Future: Send + 'static,
{
    type Service = BoxService<
        'static,
        I,
        <<M::Service as Service<I>>::Output as IntoOutcome<I>>::Success,
        <<M::Service as Service<I>>::Output as IntoOutcome<I>>::Failure,
    >;

    fn wrap(&self, service: S) -> Self::Service {
        Box::new(BoxedService {
            service: self.0.wrap(service),
        })
    }
}

pub type LocalBoxMiddleware<'a, I, O, E, S> =
    Box<dyn Middleware<I, S, Service = LocalBoxService<'a, I, O, E>>>;

pub struct LocalBoxedMiddleware<M>(pub M);

impl<'a, I, S, M> Middleware<I, S> for LocalBoxedMiddleware<M>
where
    M: Middleware<I, S>,
    S: Service<I> + 'static,
    M::Service: 'static,
    <M::Service as Service<I>>::Future: 'static,
{
    type Service = LocalBoxService<
        'static,
        I,
        <<M::Service as Service<I>>::Output as IntoOutcome<I>>::Success,
        <<M::Service as Service<I>>::Output as IntoOutcome<I>>::Failure,
    >;

    fn wrap(&self, service: S) -> Self::Service {
        Box::new(LocalBoxedService {
            service: self.0.wrap(service),
        })
    }
}
