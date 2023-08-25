use core::future::Future;
use dale::{filters::One, IntoService, Outcome, Service, ServiceExt};
use dale_http::Request;

use crate::{
    filters::{data, id, query},
    model::{Model, Query},
    Data,
};

pub struct List<M>
where
    M: Model,
{
    model: M,
    default_query: Option<M::Query>,
}

impl<M: Model> List<M> {
    pub fn new(model: M) -> List<M> {
        List {
            model,
            default_query: None,
        }
    }
}

impl<M> List<M>
where
    M: Model + Clone,
{
    pub fn service<B>(
        self,
    ) -> impl Service<
        Request<B>,
        Output = Outcome<(Request<B>, One<Vec<M::Output>>), dale_http::Error, Request<B>>,
        Future = impl Future + Send,
    > + Clone
    where
        B: Send,
        M: Model + 'static + Send,
        M::Query: Send + Clone,
        M::Error: std::error::Error + Send + Sync + 'static,
        <M::Query as Query>::Error: Send + Sync + 'static,
    {
        dale::filters::state(self.model)
            .and(query::<M, B>(self.default_query))
            .and_then(|model: M, query: M::Query| async move {
                let future = model.list(&query);
                let list = future.await.map_err(dale_http::Error::new)?;
                dale_http::Result::Ok(list)
            })
            .err_into()
    }
}

pub struct Create<M>
where
    M: Model,
{
    model: M,
}

impl<M: Model> Create<M> {
    pub fn new(model: M) -> Create<M> {
        Create { model }
    }
}

impl<M> Create<M>
where
    M: Model + Clone,
{
    pub fn service<B>(
        self,
    ) -> impl Service<
        Request<B>,
        Output = Outcome<(Request<B>, One<M::Output>), dale_http::Error, Request<B>>,
        Future = impl Future + Send,
    > + Clone
    where
        B: Send + dale_http::Body,
        B::Error: std::error::Error + Send + Sync + 'static,
        M: Model + 'static + Send,
        M::Error: std::error::Error + Send + Sync + 'static,
        M::Data: Send,
        <M::Data as Data>::Error: Send + Sync + 'static,
    {
        dale::filters::state(self.model)
            .and(data::<M, B>())
            .and_then(|model: M, data: M::Data| async move {
                let future = model.create(data);
                let list = future.await.map_err(dale_http::Error::new)?;
                dale_http::Result::Ok(list)
            })
            .err_into()
    }
}

pub struct Retrieve<M>
where
    M: Model,
{
    model: M,
    key: String,
}

impl<M: Model> Retrieve<M> {
    pub fn new(model: M) -> Retrieve<M> {
        Retrieve {
            model,
            key: String::from("id"),
        }
    }
}

impl<M> Retrieve<M>
where
    M: Model + Clone,
{
    pub fn service<B>(
        self,
    ) -> impl Service<
        Request<B>,
        Output = Outcome<(Request<B>, One<M::Output>), dale_http::Error, Request<B>>,
        Future = impl Future + Send,
    > + Clone
    where
        B: Send + dale_http::Body,
        B::Error: std::error::Error + Send + Sync + 'static,
        M: Model + 'static + Send,
        M::Error: std::error::Error + Send + Sync + 'static,
    {
        dale::filters::state(self.model)
            .and(id(self.key))
            .and_then(|model: M, id: String| async move {
                let future = model.get(&id);
                let list = future.await.map_err(dale_http::Error::new)?;
                dale_http::Result::Ok(list)
            })
            .err_into()
    }
}
