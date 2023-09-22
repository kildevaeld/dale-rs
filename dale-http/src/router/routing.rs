use dale::{IntoOutcome, Service};
use http::{Method, Request};
use router::AsSegments;

use crate::{Error, Reply};

use super::Route;

//
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
        <S::Output as IntoOutcome<Request<B>>>::Success: Reply<B> + Send,
        <S::Output as IntoOutcome<Request<B>>>::Failure: Into<Error>;

    fn mount<'a, 'b, P, I>(&mut self, path: P, router: I) -> Result<&mut Self, P::Error>
    where
        P: AsSegments<'a> + 'a,
        I: IntoIterator<Item = router::Route<'b, Route<B>>>;

    fn extend<'a, I>(&mut self, router: I) -> &mut Self
    where
        I: IntoIterator<Item = router::Route<'a, Route<B>>>;
}
