use std::fmt;

use crate::{error::Error, service_ext::HttpServiceExt, Reply};
use dale::{
    BoxService, IntoOutcomeExt, Middleware, MiddlewareFnService, Service, ServiceExt,
    ServiceFailure, ServiceSuccess,
};
use futures_core::Future;
use http::{Method, Request, Response};

pub struct Route<B> {
    pub(super) service: BoxService<'static, Request<B>, Response<B>, Error>,
    pub(super) method: Method,
}

impl<B> fmt::Debug for Route<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .finish_non_exhaustive()
    }
}

impl<B> Route<B> {
    pub(super) fn new(
        method: Method,
        service: BoxService<'static, Request<B>, Response<B>, Error>,
    ) -> Route<B> {
        Route { service, method }
    }

    pub fn wrap<M>(mut self, middleware: M) -> Route<B>
    where
        B: Send + 'static,
        M: Middleware<Request<B>, BoxService<'static, Request<B>, Response<B>, Error>>,
        M::Service: Service<Request<B>> + Send + Sync + 'static,
        <M::Service as Service<Request<B>>>::Future: Send,
        ServiceSuccess<Request<B>, M::Service>: Reply<B> + Send,
        ServiceFailure<Request<B>, M::Service>: Into<Error>,
    {
        self.service = middleware
            .wrap(self.service)
            .err_into()
            .into_response()
            .boxed();
        self
    }
}
