mod controller;
pub mod filters;
mod handlers;
mod model;

pub use self::{handlers::*, model::*};
// use std::future::Future;

// use async_trait::async_trait;
// use dale::{filters::One, IntoOutcome, Outcome, Service, ServiceExt};
// use dale_http::{Request, RequestExt};

// pub trait Query: Sized {
//     type Error: std::error::Error;
//     fn from_request<B>(req: &Request<B>) -> Result<Self, Self::Error>;
// }

// #[async_trait]
// pub trait Model {
//     type Query: Query;
//     type Data;
//     type Error;
//     type Output;

//     async fn count(&self, query: &Self::Query) -> Result<u64, Self::Error>;
//     async fn list(&self, query: &Self::Query) -> Result<Vec<Self::Output>, Self::Error>;
//     async fn get(&self, id: &str) -> Result<Self::Output, Self::Error>;
//     async fn create(&self, data: Self::Data) -> Result<Self::Output, Self::Error>;
// }

/*

pub fn query<M, B: Send>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<M::Query>), dale_http::Error, Request<B>>,
    Future = impl Future + Send,
> + Clone
       + Copy
where
    M: Model,
    <M::Query as Query>::Error: Send + Sync + 'static,
{
    |req: Request<B>| async move {
        let ret = match M::Query::from_request(&req) {
            Ok(ret) => ret,
            Err(err) => return Err(dale_http::Error::new(err)).into_outcome(),
        };

        Ok((req, (ret,))).into_outcome()
    }
}

pub fn id<B: Send>(
    path: impl ToString,
) -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<String>), dale_http::Error, Request<B>>,
    Future = impl Future + Send,
> + Clone {
    let path = path.to_string();
    move |req: Request<B>| {
        let id = req.params().get(&path).map(|m| m.to_string());
        async move {
            match id {
                Some(id) => Outcome::Success((req, (id.to_string(),))),
                None => Outcome::Next(req),
            }
        }
    }
}

pub fn list<M: Clone, B>(
    model: M,
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
    dale::filters::state(model)
        .and(query::<M, B>())
        .and_then(|model: M, query: M::Query| async move {
            let future = model.list(&query);
            let list = future.await.map_err(dale_http::Error::new)?;
            dale_http::Result::Ok(list)
        })
        .err_into()
}

pub fn get<M: Clone, B>(
    model: M,
    key: impl ToString,
) -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<M::Output>), dale_http::Error, Request<B>>,
    Future = impl Future + Send,
> + Clone
where
    B: Send,
    M: Model + 'static + Send,
    M::Query: Send,
    M::Error: std::error::Error + Send + Sync + 'static,
    <M::Query as Query>::Error: Send + Sync + 'static,
{
    dale::filters::state(model)
        .and(id(key))
        .and(query::<M, B>())
        .and_then(|model: M, id: String, query: M::Query| async move {
            let future = model.get(&id);
            let list = future.await.map_err(dale_http::Error::new)?;
            dale_http::Result::Ok(list)
        })
        .err_into()
}

pub fn create<M: Clone, B>(
    model: M,
) -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<M::Output>), dale_http::Error, Request<B>>,
    Future = impl Future + Send,
> + Clone
where
    B: Send,
    M: Model + 'static + Send,
    M::Query: Send,
    M::Error: std::error::Error + Send + Sync + 'static,
    <M::Query as Query>::Error: Send + Sync + 'static,
{
    dale::filters::state(model)
        .and(query::<M, B>())
        .and_then(|model: M, query: M::Query| async move {
            let future = model.get(&id);
            let list = future.await.map_err(dale_http::Error::new)?;
            dale_http::Result::Ok(list)
        })
        .err_into()
}

*/
