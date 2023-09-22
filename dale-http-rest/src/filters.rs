use std::future::Future;

use dale::{filters::One, IntoOutcome, Outcome, Service, ServiceExt};
use dale_http::{Body, Request, RequestExt};

use crate::Data;

use super::model::{Model, Query};

// pub fn query<M, B: Send>(
//     default: Option<M::Query>,
// ) -> impl Service<
//     Request<B>,
//     Output = Outcome<(Request<B>, One<M::Query>), dale_http::Error, Request<B>>,
//     Future = impl Future + Send,
// > + Clone
// where
//     M: Model,
//     M::Query: Send + Clone,
//     <M::Query as Query<M>>::Error: Send + Sync + 'static,
// {
//     move |req: Request<B>| {
//         let ret = match M::Query::from_request(&req, default.as_ref()) {
//             Ok(ret) => Ok((req, (ret,))).into_outcome(),
//             Err(err) => Err(dale_http::Error::new(err)).into_outcome(),
//         };

//         async move { ret }
//     }
// }

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

pub fn data<M, B>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<M::Data>), dale_http::Error, Request<B>>,
    Future = impl Future + Send,
> + Clone
where
    B: Send + Body,
    B::Error: std::error::Error + Send + Sync + 'static,
    M: Model,
    M::Data: Send,
    <M::Data as Data>::Error: Send + Sync + 'static,
{
    |mut req: Request<B>| async move {
        match M::Data::from_request(&mut req)
            .await
            .map_err(dale_http::Error::new)
        {
            Ok(ret) => Outcome::Success((req, (ret,))),
            Err(err) => Outcome::Failure(err),
        }
    }
}
