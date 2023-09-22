use super::{route::Route, router::RouterService, routing::Routing, Params, Router};
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

impl<B, M> Routing<B> for DecoratedRouter<B, M>
where
    B: Send + 'static,
    M: Middleware<Request<B>, BoxService<'static, Request<B>, Response<B>, Error>> + Clone,
    M::Service: Service<Request<B>> + Send + Sync + 'static,
    <M::Service as Service<Request<B>>>::Future: Send,
    ServiceSuccess<Request<B>, M::Service>: Reply<B> + Send,
    ServiceFailure<Request<B>, M::Service>: Into<Error>,
{
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
        let service_box = service
            .then(
                |resp: <S::Output as IntoOutcome<Request<B>>>::Success| async move {
                    let resp = resp.into_response();
                    Result::<_, Error>::Ok(resp)
                },
            )
            .err_into()
            .boxed();

        self.router
            .register(method, path, self.middleware.wrap(service_box))?;

        Ok(self)
    }

    fn mount<'a, 'b, P, I>(&mut self, path: P, router: I) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
        I: IntoIterator<Item = router::Route<'b, Route<B>>>,
    {
        self.router.mount(
            path,
            router
                .into_iter()
                .map(|route| route.map(|handle| handle.wrap(self.middleware.clone()))),
        )?;

        Ok(self)
    }

    fn extend<'a, I>(&mut self, router: I) -> &mut Self
    where
        I: IntoIterator<Item = router::Route<'a, Route<B>>>,
    {
        self.router.extend(
            router
                .into_iter()
                .map(|route| route.map(|handle| handle.wrap(self.middleware.clone()))),
        );

        self
    }

    fn wrap<M1>(self, middleware: M1) -> DecoratedRouter<B, M1>
    where
        Self: Sized,
        B: Send + 'static,
        M1: Middleware<Request<B>, BoxService<'static, Request<B>, Response<B>, Error>> + Clone,
        M1::Service: Service<Request<B>> + Send + Sync + 'static,
        <M1::Service as Service<Request<B>>>::Future: Send,
        ServiceSuccess<Request<B>, M1::Service>: Reply<B> + Send,
        ServiceFailure<Request<B>, M1::Service>: Into<Error>,
    {
        DecoratedRouter::new(middleware, self.router)
    }
}

impl<B: Body + Send + Sync + 'static, M> IntoService<Request<B>> for DecoratedRouter<B, M> {
    type Error = Infallible;
    type Service = RouterService<B>;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        self.router.into_service()
    }
}

impl<B, M> IntoIterator for DecoratedRouter<B, M> {
    type IntoIter = router::router::IntoIter<Route<B>>;
    type Item = router::Route<'static, Route<B>>;
    fn into_iter(self) -> Self::IntoIter {
        self.router.into_iter()
    }
}
