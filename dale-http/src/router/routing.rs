use dale::{
    combinators::shared::SharedService, BoxService, IntoOutcome, Middleware, MiddlewareFn, Service,
    ServiceFailure, ServiceSuccess,
};
use futures_core::Future;
use http::{Method, Request, Response};
use router::AsSegments;

use crate::{Error, Reply};

use super::{decorated::DecoratedRouter, Route};

macro_rules! impl_method {
    ($($name: ident => $method: ident),*) => {
        $(
            fn $name<'a, P, S>(&mut self, path: P, service: S) -> Result<&mut Self, P::Error>
            where
                P: AsSegments<'a> + 'a,
                B: 'static,
                S: Service<Request<B>> + Send + Sync + 'static,
                S::Future: Send,
                ServiceSuccess<Request<B>, S>: Reply<B> + Send,
                ServiceFailure<Request<B>, S>: Into<Error>,
            {
                self.register(Method::$method, path, service)
            }
        )*
    };
}

pub trait Routing<B> {
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
        ServiceSuccess<Request<B>, S>: Reply<B> + Send,
        ServiceFailure<Request<B>, S>: Into<Error>;

    fn mount<'a, 'b, P, I>(&mut self, path: P, router: I) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
        I: IntoIterator<Item = router::Route<'b, Route<B>>>;

    fn extend<'a, I>(&mut self, router: I) -> &mut Self
    where
        I: IntoIterator<Item = router::Route<'a, Route<B>>>;

    fn wrap<M>(self, middleware: M) -> DecoratedRouter<B, M>
    where
        Self: Sized,
        B: Send + 'static,
        M: Middleware<Request<B>, BoxService<'static, Request<B>, Response<B>, Error>> + Clone,
        M::Service: Service<Request<B>> + Send + Sync + 'static,
        <M::Service as Service<Request<B>>>::Future: Send,
        ServiceSuccess<Request<B>, M::Service>: Reply<B> + Send,
        ServiceFailure<Request<B>, M::Service>: Into<Error>;

    impl_method!(
        get => GET,
        post => POST,
        put => PUT,
        patch => PATCH,
        delete => DELETE
    );
}
