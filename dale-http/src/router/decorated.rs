use super::{route::Route, routing::Routing, Params, Router};
use crate::{error::Error, Body, Outcome, Reply};
use dale::{
    boxed::BoxFuture, BoxService, IntoOutcome, IntoService, Middleware, Service, ServiceExt,
    ServiceFailure, ServiceSuccess,
};
use http::{Method, Request, Response, StatusCode};
use router::{AsSegments, Router as LibRouter};
use std::{convert::Infallible, sync::Arc};

pub struct DecoratedRouter<B, M> {
    router: Router<B>,
    middleware: M,
}

impl<B, M> DecoratedRouter<B, M> {
    pub fn new(middleware: M, router: Router<B>) -> DecoratedRouter<B, M>
    where
        B: Send + 'static,
        M: Middleware<Request<B>, BoxService<'static, Request<B>, Response<B>, Error>> + Clone,
        M::Service: Service<Request<B>> + Send + Sync + 'static,
        <M::Service as Service<Request<B>>>::Future: Send,
        ServiceSuccess<Request<B>, M::Service>: Reply<B> + Send,
        ServiceFailure<Request<B>, M::Service>: Into<Error>,
    {
        let mut new_router = Router::default();

        new_router.extend(
            router
                .into_iter()
                .map(|route| route.map(|handle| handle.wrap(middleware.clone()))),
        );

        DecoratedRouter {
            router: new_router,
            middleware,
        }
    }
}

impl<B, M> Routing<B> for DecoratedRouter<B, M> {
    fn register<'a, P, S>(
        &mut self,
        method: Method,
        path: P,
        service: S,
    ) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
        B: 'static,
        S: Service<Request<B>> + Send + Sync + 'static,
        S::Future: Send,
        <S::Output as IntoOutcome<Request<B>>>::Success: Reply<B> + Send,
        <S::Output as IntoOutcome<Request<B>>>::Failure: Into<Error>,
    {
        todo!()
    }

    fn mount<'a, 'b, P, I>(&mut self, path: P, router: I) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
        I: IntoIterator<Item = router::Route<'b, Route<B>>>,
    {
        todo!()
    }

    fn extend<'a, I>(&mut self, router: I) -> &mut Self
    where
        I: IntoIterator<Item = router::Route<'a, Route<B>>>,
    {
        todo!()
    }
}
