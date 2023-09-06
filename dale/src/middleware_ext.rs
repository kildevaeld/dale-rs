use crate::{Middleware, Service};

pub trait MiddlewareExt<R, S>: Middleware<R, S>
where
    S: Service<R>,
{
    #[cfg(feature = "alloc")]
    fn boxed(
        self,
    ) -> crate::boxed::BoxMiddleware<
        'static,
        R,
        crate::MiddlewareSuccess<R, Self, S>,
        crate::MiddlewareFailure<R, Self, S>,
        S,
    >
    where
        Self: Sized + Send + Sync + 'static,
        S: Sync + Send + 'static,
        Self::Service: Send + Sync + 'static,
        <Self::Service as Service<R>>::Future: Send + 'static,
    {
        Box::new(crate::boxed::BoxedMiddleware(self))
    }

    #[cfg(feature = "alloc")]
    fn boxed_local(
        self,
    ) -> crate::boxed::LocalBoxMiddleware<
        'static,
        R,
        crate::MiddlewareSuccess<R, Self, S>,
        crate::MiddlewareFailure<R, Self, S>,
        S,
    >
    where
        Self: Sized + 'static,
        S: 'static,
        Self::Service: 'static,
        <Self::Service as Service<R>>::Future: 'static,
    {
        Box::new(crate::boxed::LocalBoxedMiddleware(self))
    }

    fn and<M>(self, middleware: M) -> And<Self, M>
    where
        Self: Sized,
        M: Middleware<R, Self::Service>,
    {
        And {
            left: self,
            right: middleware,
        }
    }
}

impl<M, R, S> MiddlewareExt<R, S> for M
where
    M: Middleware<R, S>,
    S: Service<R>,
{
}

pub struct And<L, R> {
    left: L,
    right: R,
}

impl<L, R, S> Middleware<R, S> for And<L, R>
where
    L: Middleware<R, S>,
    R: Middleware<R, L::Service>,
    S: Service<R>,
{
    type Service = R::Service;

    fn wrap(&self, service: S) -> Self::Service {
        self.right.wrap(self.left.wrap(service))
    }
}
