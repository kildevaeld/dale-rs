use dale::{
    filters::One, IntoOutcome, IntoService, Service, ServiceExt, ServiceFailure, ServiceSuccess,
};
use dale_http::{
    reply,
    router::{AsSegments, IntoIter, Router},
    Body, Request,
};

use crate::{route::RestRoute, Model, RouteSet};

#[derive(Debug, Default)]
pub struct RestRouter<B> {
    router: Router<B>,
}

impl<B> RestRouter<B> {
    pub fn register<'a, P, S, O>(
        &mut self,
        route: RestRoute<P, S>,
    ) -> Result<&mut Self, <P as AsSegments<'a>>::Error>
    where
        B: Body + Send + 'static,
        S: Service<Request<B>> + Send + Sync + 'static,
        S::Future: Send,
        S::Output: IntoOutcome<Request<B>, Success = (Request<B>, One<O>)>,
        ServiceFailure<Request<B>, S>: Into<dale_http::Error>,
        O: serde::ser::Serialize + Send + 'static,
        P: AsSegments<'a> + 'a,
    {
        let service = route.service;

        self.router.register(
            route.method.into(),
            route.segments,
            service.map(reply::json),
        )?;

        Ok(self)
    }

    pub fn extend<M: Model>(mut self, routes: RouteSet<M>) {}
}

impl<B> IntoIterator for RestRouter<B> {
    type IntoIter = IntoIter<B>;
    type Item = <Router<B> as IntoIterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        self.router.into_iter()
    }
}

impl<B: Body + Send + Sync + 'static> IntoService<Request<B>> for RestRouter<B> {
    type Error = <Router<B> as IntoService<Request<B>>>::Error;
    type Service = <Router<B> as IntoService<Request<B>>>::Service;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        self.router.into_service()
    }
}
