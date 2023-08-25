use core::future::Future;
use dale::{filters::One, IntoService, Outcome, Service, ServiceExt};
use dale_http::Request;

use crate::{
    filters::query,
    model::{Model, Query},
};

pub struct List<M> {
    model: M,
}

impl<M> List<M>
where
    M: Model + Clone,
{
    fn service<B>(
        self,
    ) -> impl Service<
        Request<B>,
        Output = Outcome<(Request<B>, One<Vec<M::Output>>), dale_http::Error, Request<B>>,
        Future = impl Future + Send,
    > + Clone
    where
        B: Send,
        M: Model + 'static + Send,
        M::Query: Send,
        M::Error: std::error::Error + Send + Sync + 'static,
        <M::Query as Query>::Error: Send + Sync + 'static,
    {
        dale::filters::state(self.model)
            .and(query::<M, B>())
            .and_then(|model: M, query: M::Query| async move {
                let future = model.list(&query);
                let list = future.await.map_err(dale_http::Error::new)?;
                dale_http::Result::Ok(list)
            })
            .err_into()
    }
}
