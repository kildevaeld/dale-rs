use super::{decorated::DecoratedRouter, route::Route, routing::Routing, Params};
use crate::{error::Error, Body, Outcome, Reply};
use dale::{
    boxed::BoxFuture, BoxService, IntoOutcome, IntoService, Middleware, Service, ServiceExt,
    ServiceFailure, ServiceSuccess,
};
use http::{Method, Request, Response, StatusCode};
use router::{AsSegments, Router as LibRouter};
use std::{convert::Infallible, sync::Arc};

#[derive(Debug)]
pub struct Router<B> {
    router: LibRouter<Route<B>>,
}

impl<B> Default for Router<B> {
    fn default() -> Self {
        Router::new()
    }
}

impl<B> Router<B> {
    pub fn new() -> Router<B> {
        Router {
            router: LibRouter::new(),
        }
    }
}

impl<B> Routing<B> for Router<B> {
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
            .register(path, Route::new(method, service_box))?;

        Ok(self)
    }

    fn mount<'a, 'b, P, I>(&mut self, path: P, router: I) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
        I: IntoIterator<Item = router::Route<'b, Route<B>>>,
    {
        self.router.mount(path, router)?;
        Ok(self)
    }

    fn extend<'a, I>(&mut self, router: I) -> &mut Self
    where
        I: IntoIterator<Item = router::Route<'a, Route<B>>>,
    {
        self.router.extend(router);
        self
    }

    fn wrap<M>(self, middleware: M) -> DecoratedRouter<B, M>
    where
        Self: Sized,
        B: Send + 'static,
        M: Middleware<Request<B>, BoxService<'static, Request<B>, Response<B>, Error>> + Clone,
        M::Service: Service<Request<B>> + Send + Sync + 'static,
        <M::Service as Service<Request<B>>>::Future: Send,
        ServiceSuccess<Request<B>, M::Service>: Reply<B> + Send,
        ServiceFailure<Request<B>, M::Service>: Into<Error>,
    {
        DecoratedRouter::new(middleware, self)
    }
}

impl<B: Body + Send + Sync + 'static> IntoService<Request<B>> for Router<B> {
    type Error = Infallible;
    type Service = RouterService<B>;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        Ok(RouterService {
            router: self.into(),
        })
    }
}

impl<B> IntoIterator for Router<B> {
    type IntoIter = router::router::IntoIter<Route<B>>;
    type Item = router::Route<'static, Route<B>>;
    fn into_iter(self) -> Self::IntoIter {
        self.router.into_iter()
    }
}

#[derive(Debug)]
pub struct RouterService<B> {
    router: Arc<Router<B>>,
}

impl<B> Clone for RouterService<B> {
    fn clone(&self) -> Self {
        RouterService {
            router: self.router.clone(),
        }
    }
}

impl<B: Body + Send + Sync + 'static> Service<Request<B>> for RouterService<B> {
    type Output = Outcome<B>;

    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, mut req: Request<B>) -> Self::Future {
        let router = self.router.clone();

        Box::pin(async move {
            let mut params = Params::default();
            let found = match router.router.find(req.uri().path(), &mut params) {
                Some(found) => found,
                None => return Outcome::Next(req),
            };

            let method = req.method().clone();

            let is_head = method == Method::HEAD;

            req.extensions_mut().insert(params);

            for next in found
                .iter()
                .filter(|route| route.method == method || (is_head && route.method == Method::GET))
            {
                match next.service.call(req).await.into_outcome() {
                    Outcome::Next(r) => {
                        req = r;
                    }
                    Outcome::Success(mut success) => {
                        if method != next.method && is_head {
                            *success.body_mut() = B::empty();
                            *success.status_mut() = StatusCode::NO_CONTENT;
                        }

                        return dale::Outcome::Success(success);
                    }
                    o => return o,
                }
            }

            req.extensions_mut().remove::<Params>();

            Outcome::Next(req)
        })
    }
}
