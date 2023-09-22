use dale::{IntoOutcome, Middleware, Service, ServiceExt, ServiceFailure};
use dale_http::router::{AsSegments, Segments};
use dale_http::{router::Router, Reply};

use dale_http::{Body, Request};

use crate::method::RestMethod;
use crate::router::RestRouter;

pub struct RestRoute<S, T> {
    pub segments: S,
    pub service: T,
    pub method: RestMethod,
}

macro_rules! methods {
    ($n: ident, $($name: ident : $path: literal => $type: ident),*) => {
        $(
            pub fn $name(
                $n: &str,
                service: T,
            ) -> Result<RestRoute<Segments, T>, <String as AsSegments<'static>>::Error> {
                Self::register(format!($path), RestMethod::$type, service)
            }
        )*
    };
}

impl<T> RestRoute<Segments<'static>, T> {
    pub fn create(
        name: &str,
        service: T,
    ) -> Result<RestRoute<Segments, T>, <String as AsSegments<'static>>::Error> {
        Self::register(format!("/{name}"), RestMethod::Create, service)
    }

    methods!(name, list : "/{name}" => List, retrieve : "/{name}/:{name}-id" => Retrieve);

    pub fn register<'a, S: AsSegments<'a>>(
        path: S,
        method: RestMethod,
        service: T,
    ) -> Result<RestRoute<Segments<'static>, T>, <S as AsSegments<'a>>::Error> {
        Ok(RestRoute {
            segments: Segments::new(
                path.as_segments()?
                    .into_iter()
                    .map(|m| m.to_static())
                    .collect(),
            ),
            service,
            method,
        })
    }
}

impl<S, T> RestRoute<S, T> {
    pub fn wrap<M, B>(self, middleware: M) -> RestRoute<S, M::Service>
    where
        T: Service<Request<B>> + Send + Sync + 'static,
        M: Middleware<Request<B>, T>,
        M::Service: Service<B, Output = T::Output>,
    {
        let service = self.service.wrap(middleware);
        RestRoute {
            segments: self.segments,
            service,
            method: self.method,
        }
    }

    pub fn attach<'a, B, O>(
        self,
        router: &mut RestRouter<B>,
    ) -> Result<(), <S as AsSegments<'a>>::Error>
    where
        B: Body + Send + 'static,
        T: Service<Request<B>> + Send + Sync + 'static,
        T::Future: Send,
        T::Output: IntoOutcome<Request<B>, Success = (Request<B>, (O,))>,
        ServiceFailure<Request<B>, T>: Into<dale_http::Error>,
        O: serde::ser::Serialize + Send + 'static,
        S: AsSegments<'a> + 'a,
    {
        router.register(self)?;
        Ok(())
    }
}
