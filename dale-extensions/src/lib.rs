pub trait Extensions {
    fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T>;
    fn get<T: Send + Sync + 'static>(&self) -> Option<&T>;
    fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T>;
    fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T>;
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
}

pub trait Extensible {
    type Extensions: Extensions;

    fn extensions(&self) -> &Self::Extensions;
    fn extensions_mut(&mut self) -> &mut Self::Extensions;
}

pub mod filters {

    use core::{convert::Infallible, future::Future, marker::PhantomData};
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use dale::{Outcome, Service};
    use pin_project_lite::pin_project;

    use crate::{Extensible, Extensions};

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
        type Output = Outcome<T, Infallible, R>;
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
        type Output = Outcome<T, Infallible, R>;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.as_mut().project();
            match this.req.as_ref().expect("request").extensions().get::<T>() {
                Some(ret) => Poll::Ready(Outcome::Success(ret.clone())),
                None => Poll::Ready(Outcome::Next(this.req.take().unwrap())),
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
