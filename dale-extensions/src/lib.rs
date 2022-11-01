#![no_std]

use dale::{Middleware, Service};

pub trait Extensions {
    fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T>;
    fn get<T: Send + Sync + 'static>(&self) -> Option<&T>;
    fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T>;
    fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T>;
    fn contains<T: Send + Sync + 'static>(&self) -> bool;
}

// #[cfg(feature = "extensions")]
// impl Extensions for extensions::Extensions {
//     fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
//         self.insert(val)
//     }

//     fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
//         self.get()
//     }

//     fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
//         self.get_mut()
//     }

//     fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
//         self.remove()
//     }
// }

#[cfg(feature = "extensions")]
impl Extensions for extensions::concurrent::Extensions {
    fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.insert(val)
    }

    fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.get()
    }

    fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.get_mut()
    }

    fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.remove()
    }

    fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.get::<T>().is_some()
    }
}

#[cfg(feature = "http")]
impl Extensions for http::Extensions {
    fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.insert(val)
    }

    fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.get()
    }

    fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.get_mut()
    }

    fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.remove()
    }

    fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.get::<T>().is_some()
    }
}

#[cfg(feature = "http")]
impl<B> Extensible for http::Request<B> {
    type Extensions = http::Extensions;

    fn extensions(&self) -> &Self::Extensions {
        self.extensions()
    }

    fn extensions_mut(&mut self) -> &mut Self::Extensions {
        self.extensions_mut()
    }
}

pub trait Extensible {
    type Extensions: Extensions;

    fn extensions(&self) -> &Self::Extensions;
    fn extensions_mut(&mut self) -> &mut Self::Extensions;
}

pub struct StateMiddleware<S> {
    state: S,
}

impl<S> StateMiddleware<S> {
    pub fn new(state: S) -> StateMiddleware<S> {
        StateMiddleware { state }
    }
}

impl<S, T, R> Middleware<R, T> for StateMiddleware<S>
where
    S: Clone + Send + Sync + 'static,
    T: Service<R>,
    R: Extensible,
{
    type Service = State<T, S>;

    fn wrap(&self, service: T) -> Self::Service {
        State {
            state: self.state.clone(),
            service,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct State<T, S> {
    state: S,
    service: T,
}

impl<T, S, R> Service<R> for State<T, S>
where
    T: Service<R>,
    S: Clone + Send + Sync + 'static,
    R: Extensible,
{
    type Output = T::Output;

    type Future = T::Future;

    fn call(&self, mut req: R) -> Self::Future {
        req.extensions_mut().insert(self.state.clone());
        self.service.call(req)
    }
}

pub mod filters {

    use crate::{Extensible, Extensions};
    use core::{convert::Infallible, future::Future, marker::PhantomData};
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };
    use dale::{Outcome, Service};
    use pin_project_lite::pin_project;

    pub struct ExtensionService<T>(PhantomData<T>);

    impl<T> Clone for ExtensionService<T> {
        fn clone(&self) -> Self {
            ExtensionService(PhantomData)
        }
    }

    impl<T> Copy for ExtensionService<T> {}

    impl<T, R> Service<R> for ExtensionService<T>
    where
        R: Extensible,
        T: Sync + Send + Clone + 'static,
    {
        type Output = Outcome<(R, (T,)), Infallible, R>;
        type Future = ExtensionServiceFuture<T, R>;

        fn call(&self, req: R) -> Self::Future {
            ExtensionServiceFuture {
                req: Some(req),
                _t: PhantomData,
            }
        }
    }

    pin_project! {
        pub struct ExtensionServiceFuture<T, R> {
            req: Option<R>,
            _t: PhantomData<T>,
        }
    }

    impl<T, R> Future for ExtensionServiceFuture<T, R>
    where
        R: Extensible,
        T: Sync + Send + Clone + 'static,
    {
        type Output = Outcome<(R, (T,)), Infallible, R>;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.as_mut().project();
            let req = this.req.take().unwrap();

            match req.extensions().get::<T>() {
                Some(ret) => {
                    let ret = ret.clone();
                    Poll::Ready(Outcome::Success((req, (ret,))))
                }
                None => Poll::Ready(Outcome::Next(req)),
            }
        }
    }

    pub fn ext<T>() -> ExtensionService<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        ExtensionService(PhantomData)
    }
}
