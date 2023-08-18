use std::{
    future::Future,
    sync::Arc,
    task::{ready, Poll},
};

use core::marker::PhantomData;
use dale::{IntoOutcome, Middleware, Service};
use dale_http::{prelude::Set, HeaderValue, Outcome, Reply, Request};
use parking_lot::Mutex;

use crate::cookie_jar::CookieJar;

#[derive(Debug, Clone, Copy)]
pub struct Cookies;

impl<T, B> Middleware<Request<B>, T> for Cookies
where
    T: Service<Request<B>> + Clone,
    <T::Output as IntoOutcome<Request<B>>>::Success: Reply<B>,
    <T::Output as IntoOutcome<Request<B>>>::Failure: Into<dale_http::Error>,
{
    type Service = CookiesService<T>;
    fn wrap(&self, service: T) -> Self::Service {
        CookiesService { service }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CookiesService<T> {
    service: T,
}

impl<T, B> Service<Request<B>> for CookiesService<T>
where
    T: Service<Request<B>> + Clone,
    <T::Output as IntoOutcome<Request<B>>>::Success: Reply<B>,
    <T::Output as IntoOutcome<Request<B>>>::Failure: Into<dale_http::Error>,
{
    type Output = Outcome<B>;

    type Future = CookieServiceFuture<T, B>;

    fn call(&self, req: Request<B>) -> Self::Future {
        CookieServiceFuture {
            state: CookieServiceFutureState::Init {
                service: self.service.clone(),
                request: Some(req),
            },
            _body: PhantomData,
        }
    }
}

pin_project_lite::pin_project! {
    #[project = StateProj]
    enum CookieServiceFutureState<T, B> where T: Service<Request<B>> {
        Init {
            service: T,
            request: Option<Request<B>>
        },
        Service {
            #[pin]
            future: T::Future,
            cookie_jar: CookieJar
        },
        Done
    }
}

pin_project_lite::pin_project! {

    pub struct CookieServiceFuture<T, B> where T: Service<Request<B>> {
        #[pin]
        state: CookieServiceFutureState<T, B>,
        _body: PhantomData<B>
    }
}

impl<T, B> Future for CookieServiceFuture<T, B>
where
    T: Service<Request<B>>,
    <T::Output as IntoOutcome<Request<B>>>::Success: Reply<B>,
    <T::Output as IntoOutcome<Request<B>>>::Failure: Into<dale_http::Error>,
{
    type Output = Outcome<B>;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.as_mut().project();

        loop {
            let state = this.state.as_mut().project();

            match state {
                StateProj::Init { service, request } => {
                    let mut req = request.take().unwrap();

                    let cookie_jar = parse_cookies(&req);

                    req.extensions_mut().insert(cookie_jar.clone());

                    let future = service.call(req);
                    this.state
                        .set(CookieServiceFutureState::Service { future, cookie_jar });
                }
                StateProj::Service { future, cookie_jar } => {
                    let ret = match ready!(future.poll(cx)).into_outcome() {
                        dale::Outcome::Next(next) => Outcome::Next(next),
                        dale::Outcome::Success(ret) => {
                            Outcome::Success(ret.into_response().set(&*cookie_jar))
                        }
                        dale::Outcome::Failure(err) => Outcome::Failure(err.into()),
                    };

                    this.state.set(CookieServiceFutureState::Done);

                    return Poll::Ready(ret);
                }
                StateProj::Done => {
                    panic!("poll after done")
                }
            };
        }
    }
}

fn parse_cookies<B>(req: &Request<B>) -> CookieJar {
    let cookie_header = req.headers().get(dale_http::http::header::COOKIE);

    let mut jar = cookie::CookieJar::new();

    if let Some(header) = cookie_header {
        let header_str = header.to_str().unwrap();
        let cookies = cookie::Cookie::split_parse(header_str);

        for cookie in cookies {
            let cookie = match cookie {
                Ok(cookie) => cookie,
                Err(err) => {
                    tracing::info!(err = ?err, "could not parse cookie");
                    continue;
                }
            };

            jar.add_original(cookie.into_owned());
        }
    }

    CookieJar(Arc::new(Mutex::new(jar)))
}
