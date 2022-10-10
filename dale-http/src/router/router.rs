use super::{route::Route, Params};
use crate::{error::Error, Body, Outcome, Reply};
use dale::{boxed::BoxFuture, IntoOutcome, Service, ServiceExt};
use http::{Method, Request, StatusCode};
use router::{AsSegments, Router as LibRouter};
use std::sync::Arc;

#[derive(Debug)]
pub struct Router<B> {
    router: LibRouter<Route<B>>,
}

impl<B> Default for Router<B> {
    fn default() -> Self {
        Router::new()
    }
}

macro_rules! impl_method {
    ($($name: ident => $method: ident),*) => {
        $(
            pub fn $name<'a, P, S>(&mut self, path: P, service: S) -> Result<&mut Self, P::Error>
            where
                P: AsSegments<'a> + 'a,
                B: 'static,
                S: Service<Request<B>> + Send + Sync + 'static,
                S::Future: Send,
                <S::Output as IntoOutcome<Request<B>>>::Success: Reply<B> + Send,
                <S::Output as IntoOutcome<Request<B>>>::Failure: Into<Error>,
            {
                self.register(Method::$method, path, service)
            }
        )*
    };
}

impl<B> Router<B> {
    pub fn new() -> Router<B> {
        Router {
            router: LibRouter::new(),
        }
    }

    pub fn register<'a, P, S>(
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

    pub fn mount<'a, P>(&mut self, path: P, router: Router<B>) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
    {
        self.router.mount(path, router.router)?;
        Ok(self)
    }

    pub fn extend(&mut self, router: Router<B>) -> &mut Self {
        self.router.extend(router.router);
        self
    }

    pub fn into_service(self) -> RouterService<B> {
        RouterService {
            router: Arc::new(self),
        }
    }

    impl_method!(
        get => GET,
        post => POST,
        put => PUT,
        patch => PATCH,
        delete => DELETE
    );
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
