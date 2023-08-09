use futures_core::{future::BoxFuture, Future};
use tokio::task::JoinError;

use crate::executor::Executor;

use super::Tokio;

impl Executor for Tokio {
    type Error = JoinError;
    fn spawn<F: Future + 'static + Send>(future: F)
    where
        F::Output: Send,
    {
        tokio::spawn(future);
    }

    fn unblock<R, F>(ret: F) -> BoxFuture<'static, Result<R, JoinError>>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        Box::pin(async move {
            
            tokio::task::spawn_blocking(ret).await
        })
    }

    // fn block_on<F: Future>(&self, future: F) -> F::Output {
    //     futures_lite::future::block_on(future)
    // }
}
