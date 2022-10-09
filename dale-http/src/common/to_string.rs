use super::ToBytes;
use crate::{
    error::{Error, KnownError},
    Body,
};
use futures_core::{ready, Future};
use pin_project_lite::pin_project;
use std::task::Poll;

pin_project! {
    pub struct ToText< B> where B: Body {
        #[pin]
        future: ToBytes<B>
    }
}

impl<B> ToText<B>
where
    B: Body,
{
    pub fn new(body: B) -> ToText<B> {
        ToText {
            future: ToBytes::new(body),
        }
    }
}

impl<B> Future for ToText<B>
where
    B: Body,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    type Output = Result<String, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        let bytes = match ready!(this.future.poll(cx)) {
            Ok(bytes) => bytes,
            Err(err) => return Poll::Ready(Err(Error::new(err))),
        };

        let text = match String::from_utf8(bytes.to_vec()) {
            Ok(ret) => ret,
            Err(err) => return Poll::Ready(Err(KnownError::Utf8(err.utf8_error()).into())),
        };

        return Poll::Ready(Ok(text));
    }
}
