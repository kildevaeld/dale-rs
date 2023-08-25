use std::future::Future;

use async_trait::async_trait;
use dale::{filters::One, IntoOutcome, Outcome, Service, ServiceExt};
use dale_http::{Request, RequestExt};

use super::model::{Model, Query};

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
