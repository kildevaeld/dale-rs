use crate::into_outcome::IntoOutcome;
use crate::service::Service;
use crate::{ServiceFailure, ServiceSuccess};
use core::future::Future;
use core::marker::PhantomData;

pub type MiddlewareSuccess<I, M, S> = ServiceSuccess<I, <M as Middleware<I, S>>::Service>;
pub type MiddlewareFailure<I, M, S> = ServiceFailure<I, <M as Middleware<I, S>>::Service>;

pub trait Middleware<R, T: Service<R>> {
    type Service: Service<R>;
    fn wrap(&self, service: T) -> Self::Service;
}

pub struct MiddlewareFn<R, F, T> {
    cb: F,
    _r: PhantomData<R>,
    _t: PhantomData<T>,
}

impl<R, F: Clone, T> Clone for MiddlewareFn<R, F, T> {
    fn clone(&self) -> Self {
        MiddlewareFn {
            cb: self.cb.clone(),
            _r: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<R, F: Copy, T> Copy for MiddlewareFn<R, F, T> {}

unsafe impl<R, F: Send, T> Send for MiddlewareFn<R, F, T> {}

unsafe impl<R, F: Sync, T> Sync for MiddlewareFn<R, F, T> {}

impl<R, F, T> MiddlewareFn<R, F, T> {
    pub fn new(cb: F) -> MiddlewareFn<R, F, T> {
        MiddlewareFn {
            cb,
            _r: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<R, F, T, U, O> Middleware<R, T> for MiddlewareFn<R, F, T>
where
    T: Service<R> + Clone,
    F: Clone + Fn(T, R) -> U,
    U: Future<Output = O>,
    O: IntoOutcome<R>,
{
    type Service = MiddlewareFnService<R, F, T>;
    fn wrap(&self, service: T) -> Self::Service {
        MiddlewareFnService {
            service,
            cb: self.cb.clone(),
            _a: PhantomData,
        }
    }
}

pub struct MiddlewareFnService<R, F, T> {
    service: T,
    cb: F,
    _a: PhantomData<R>,
}

impl<R, F, T, U, O> Service<R> for MiddlewareFnService<R, F, T>
where
    T: Service<R> + Clone,
    F: Fn(T, R) -> U,
    U: Future<Output = O>,
    O: IntoOutcome<R>,
{
    type Output = O;
    type Future = U;

    fn call(&self, req: R) -> Self::Future {
        (self.cb)(self.service.clone(), req)
    }
}

impl<R, F: Clone, T: Clone> Clone for MiddlewareFnService<R, F, T> {
    fn clone(&self) -> Self {
        MiddlewareFnService {
            cb: self.cb.clone(),
            service: self.service.clone(),
            _a: PhantomData,
        }
    }
}

impl<R, F: Copy, T: Copy> Copy for MiddlewareFnService<R, F, T> {}

#[derive(Debug, Clone, Copy)]
pub struct Passthrough;

impl<R, S> Middleware<R, S> for Passthrough
where
    S: Service<R>,
{
    type Service = S;
    fn wrap(&self, service: S) -> Self::Service {
        service
    }
}
