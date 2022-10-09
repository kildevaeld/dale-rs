#[cfg(feature = "alloc")]
use crate::boxed::{Box, BoxService, BoxedService, LocalBoxService, LocalBoxedService};
use crate::{
    combinators::{AndThen, ErrInto, MapErr, Or, Then},
    filters::{And, Combine, Extract, Func, Map, Tuple},
    into_outcome::IntoOutcome,
    middleware::{Middleware, MiddlewareFn, MiddlewareFnService},
    service::Service,
    types::MapFunc,
};
use core::future::Future;
use futures_core::TryFuture;

pub trait ServiceExt<T>: Service<T> {
    fn or<O: Service<T>>(self, service: O) -> Or<Self, O, T>
    where
        Self: Sized,
    {
        Or::new(self, service)
    }

    fn then<F>(self, then: F) -> Then<Self, F>
    where
        Self: Sized,
        F: MapFunc<
                Result<
                    <Self::Output as IntoOutcome<T>>::Success,
                    <Self::Output as IntoOutcome<T>>::Failure,
                >,
            > + Clone,
        F::Output: TryFuture,
    {
        Then::new(self, then)
    }

    fn and_then<F>(self, then: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: MapFunc<<Self::Output as IntoOutcome<T>>::Success> + Clone,
        F::Output: TryFuture,
    {
        AndThen::new(self, then)
    }

    // Middlewares

    fn wrap<M>(self, middleware: M) -> M::Service
    where
        M: Middleware<T, Self>,
        Self: Sized,
    {
        middleware.wrap(self)
    }

    fn wrap_fn<F, U>(self, middleware: F) -> MiddlewareFnService<T, F, Self>
    where
        Self: Sized + Clone,
        F: Clone + Fn(Self, T) -> U,
        U: Future,
        U::Output: IntoOutcome<T>,
    {
        self.wrap(MiddlewareFn::new(middleware))
    }

    // Error handling

    fn map_err<F, E>(self, func: F) -> MapErr<F, Self, E>
    where
        Self: Sized,
        F: Fn(<Self::Output as IntoOutcome<T>>::Failure) -> E + Clone,
    {
        MapErr::new(self, func)
    }

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Failure: Into<E>,
    {
        ErrInto::new(self)
    }

    // Filters

    fn and<F>(self, other: F) -> And<Self, F>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Success: Extract<T>,
        <<<Self::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract as Tuple>::HList:
            Combine<
                <<<F::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract as Tuple>::HList,
            >,
        F: Service<T> + Clone,
        <F::Output as IntoOutcome<T>>::Success: Extract<T>,
    {
        And::new(self, other)
    }

    fn map<F>(self, fun: F) -> Map<Self, F>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Success: Extract<T>,
        F: Func<<<Self::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract> + Clone,
    {
        Map::new(self, fun)
    }

    // Boxing

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn boxed(
        self,
    ) -> BoxService<
        'static,
        T,
        <Self::Output as IntoOutcome<T>>::Success,
        <Self::Output as IntoOutcome<T>>::Failure,
    >
    where
        Self: Sized + 'static + Send + Sync,
        Self::Future: 'static + Send,
    {
        Box::new(BoxedService::new(self))
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn boxed_local(
        self,
    ) -> LocalBoxService<
        'static,
        T,
        <Self::Output as IntoOutcome<T>>::Success,
        <Self::Output as IntoOutcome<T>>::Failure,
    >
    where
        Self: Sized + 'static,
        Self::Future: 'static,
    {
        Box::new(LocalBoxedService::new(self))
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn shared(self) -> crate::combinators::shared::SharedService<Self>
    where
        Self: Sized,
    {
        crate::combinators::shared::SharedService::new(self)
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn shared_local(self) -> crate::combinators::shared::LocalSharedService<Self>
    where
        Self: Sized,
    {
        crate::combinators::shared::LocalSharedService::new(self)
    }
}

impl<T, I> ServiceExt<I> for T where T: Service<I> {}
