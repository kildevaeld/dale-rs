use futures_core::{future::BoxFuture, Future};

pub trait Executor: Send + Sync {
    type Error;
    fn spawn<F: Future + 'static + Send>(&self, future: F)
    where
        F::Output: Send;

    fn unblock<R, F>(&self, ret: F) -> BoxFuture<'static, Result<R, Self::Error>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static;

    // fn block_on<F: Future>(&self, future: F) -> F::Output;
}
