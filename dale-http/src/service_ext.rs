use core::marker::PhantomData;
use dale::{IntoOutcome, Service, ServiceFailure, ServiceSuccess};
use futures_core::{ready, Future};
use http::Request;

use crate::{Error, Outcome, Reply};

pub trait HttpServiceExt<B>: Service<Request<B>> {
    fn into_response(self) -> IntoResponseService<Self>
    where
        Self: Sized,
    {
        IntoResponseService(self)
    }
}

impl<S, B> HttpServiceExt<B> for S where S: Service<Request<B>> {}

#[derive(Debug, Clone, Copy)]
pub struct IntoResponseService<S>(S);

pin_project_lite::pin_project! {
    pub struct IntoResponseFuture<S, B> where S: Service<Request<B>> {
        #[pin]
        future: S::Future,
        _body: PhantomData<B>
    }
}

impl<S, B> Future for IntoResponseFuture<S, B>
where
    S: Service<Request<B>>,
    ServiceFailure<Request<B>, S>: Into<Error>,
    ServiceSuccess<Request<B>, S>: Reply<B>,
{
    type Output = Outcome<B>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        std::task::Poll::Ready(
            ready!(this.future.poll(cx))
                .into_outcome()
                .err_into()
                .map(|m| m.into_response()),
        )
    }
}
