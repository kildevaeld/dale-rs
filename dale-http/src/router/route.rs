use std::fmt;

use crate::error::Error;
use dale::BoxService;
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
}
