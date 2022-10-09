mod and;
mod generic;
mod map;

use crate::{Outcome, Service};
use core::convert::Infallible;
use futures_core::Future;

pub fn any<T: Send>(
) -> impl Service<T, Future = impl Future + Send, Output = Outcome<(T, ()), Infallible, T>> + Copy {
    |req: T| async move { Outcome::Success((req, ())) }
}

pub fn state<T: Send, S: Send + Clone + 'static>(
    state: S,
) -> impl Service<T, Future = impl Future + Send, Output = Outcome<(T, (S,)), Infallible, T>> + Clone
{
    move |req| {
        let state = state.clone();
        async move { Outcome::Success((req, (state,))) }
    }
}

pub use self::{and::*, generic::*, map::*};
