use crate::{
    body::Body,
    error::Error,
    modifier::{Set, With},
    modifiers::{Header, Redirect},
    Reply,
};
use dale::{filters::One, IntoOutcome, Outcome, Service};
use either::Either;
use http::{header, HeaderValue, Request, Response, StatusCode, Uri};
use std::convert::Infallible;

impl<B> Reply<B> for Response<B> {
    fn into_response(self) -> Response<B> {
        self
    }
}

impl<T, B> Reply<B> for One<T>
where
    T: Reply<B>,
{
    fn into_response(self) -> Response<B> {
        self.0.into_response()
    }
}

impl<T, B> Reply<B> for (Request<B>, One<T>)
where
    T: Reply<B>,
{
    fn into_response(self) -> Response<B> {
        self.1.into_response()
    }
}

impl<V, E, B> Reply<B> for Result<V, E>
where
    V: Reply<B>,
    E: Reply<B>,
{
    fn into_response(self) -> Response<B> {
        match self {
            Ok(v) => v.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl<T, U, B> Reply<B> for Either<T, U>
where
    T: Reply<B>,
    U: Reply<B>,
{
    #[inline(always)]
    fn into_response(self) -> Response<B> {
        match self {
            Either::Left(a) => a.into_response(),
            Either::Right(b) => b.into_response(),
        }
    }
}

// Text

#[derive(Clone)]
pub struct Text<S> {
    body: S,
}

impl<S: ToString + Clone, B: Body> Service<Request<B>> for Text<S> {
    type Output = Outcome<Text<S>, Error, Request<B>>;
    type Future = std::future::Ready<Self::Output>;
    fn call(&self, _req: Request<B>) -> Self::Future {
        std::future::ready(Outcome::Success(self.clone()))
    }
}

#[allow(clippy::declare_interior_mutable_const)]
const MIME_TEXT: HeaderValue = HeaderValue::from_static("text/plain");
#[allow(clippy::declare_interior_mutable_const)]
const MIME_HTML: HeaderValue = HeaderValue::from_static("text/html");

impl<S: ToString, B: Body> Reply<B> for Text<S> {
    #[inline(always)]
    fn into_response(self) -> Response<B> {
        Response::with(StatusCode::OK)
            .set(self.body.to_string())
            .set(Header(header::CONTENT_TYPE, MIME_TEXT))
    }
}

impl<'a, B: Body> Reply<B> for &'a str {
    #[inline(always)]
    fn into_response(self) -> Response<B> {
        Response::with(StatusCode::OK)
            .set(self)
            .set(Header(header::CONTENT_TYPE, MIME_TEXT))
    }
}

impl<B: Body> Reply<B> for String {
    #[inline(always)]
    fn into_response(self) -> Response<B> {
        Response::with(StatusCode::OK)
            .set(self)
            .set(Header(header::CONTENT_TYPE, MIME_TEXT))
    }
}

#[derive(Clone, Debug)]
pub struct Html<S> {
    body: S,
}

impl<S: ToString + Clone, B: Body> Service<Request<B>> for Html<S> {
    type Output = Outcome<Html<S>, Error, Request<B>>;
    type Future = std::future::Ready<Self::Output>;
    fn call(&self, _req: Request<B>) -> Self::Future {
        std::future::ready(Outcome::Success(self.clone()))
    }
}

impl<S: Clone, A> dale::filters::Func<A> for Html<S> {
    type Output = Self;

    fn call(&self, _args: A) -> Self::Output {
        self.clone()
    }
}

impl<S: ToString, B: Body> Reply<B> for Html<S> {
    #[inline(always)]
    fn into_response(self) -> Response<B> {
        Response::with(StatusCode::OK)
            .set(self.body.to_string())
            .set(Header(header::CONTENT_TYPE, MIME_HTML))
    }
}

impl<S: ToString, B: Body> IntoOutcome<Request<B>> for Html<S> {
    type Success = Response<B>;

    type Failure = Infallible;

    fn into_outcome(self) -> dale::Outcome<Self::Success, Self::Failure, Request<B>> {
        Outcome::Success(self.into_response())
    }
}

pub fn html<S: ToString>(body: S) -> Html<S> {
    Html { body }
}

pub fn text<S: ToString>(body: S) -> Text<S> {
    Text { body }
}

pub fn redirect<B: Body>(uri: impl Into<Uri>) -> Response<B> {
    Response::with(Redirect(uri.into()))
}
