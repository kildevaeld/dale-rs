use crate::{into_outcome::IntoOutcome, Failure, Success};
use core::future::Future;

pub type ServiceSuccess<I, S> = Success<I, <S as Service<I>>::Output>;
pub type ServiceFailure<I, S> = Failure<I, <S as Service<I>>::Output>;

pub trait Service<T> {
    type Output: IntoOutcome<T>;
    type Future: Future<Output = Self::Output>;
    fn call(&self, req: T) -> Self::Future;
}

impl<T, F, U> Service<T> for F
where
    F: Fn(T) -> U,
    U: Future,
    U::Output: IntoOutcome<T>,
{
    type Output = U::Output;

    type Future = U;

    fn call(&self, req: T) -> Self::Future {
        (self)(req)
    }
}

pub struct ServiceFn<F> {
    func: F,
}

impl<T, F, U> Service<T> for ServiceFn<F>
where
    F: Fn(T) -> U,
    U: Future,
    U::Output: IntoOutcome<T>,
{
    type Output = U::Output;

    type Future = U;

    fn call(&self, req: T) -> Self::Future {
        (self.func)(req)
    }
}

pub fn service<F>(service: F) -> ServiceFn<F> {
    ServiceFn { func: service }
}
