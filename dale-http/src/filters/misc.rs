use dale::{Outcome, Service};
use futures_core::Future;
use http::Request;

use crate::error::Error;

pub fn ext<S: Clone + Send + Sync + 'static, B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (Option<S>,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    |req: Request<B>| async move {
        let ext = req.extensions().get().cloned();
        Outcome::Success((req, (ext,)))
    }
}

// pub fn set_ext<S: Clone + Send + Sync + 'static, B: Send + 'static>(
//     state: S,
// ) -> impl Service<Request<B>, Output = (Request<B>, ()), Error = Error, Future = impl Future + Send>
//        + Clone {
//     move |mut req: Request<B>| {
//         req.extensions_mut().insert(state.clone());
//         async move { Ok((req, ())) }
//     }
// }
