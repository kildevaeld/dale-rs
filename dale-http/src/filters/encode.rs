use crate::{
    body::BodyExt,
    encoder::Decoder,
    error::{BoxError, Error, KnownError},
    filters, Body,
};
use dale::{Outcome, Service, ServiceExt};
use futures_core::Future;
use http::Request;
use serde::de::DeserializeOwned;
use std::{error::Error as StdError, fmt};

pub fn decode<D: Decoder, S: DeserializeOwned, B>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, (S,)), Error, Request<B>>,
> + Copy
where
    B: Body + Send + 'static,
    B::Error: Into<Error> + Send,
    D::Error: Into<BoxError>,
{
    is_content_type::<D, B>()
        .and(filters::body())
        .and_then(|body: B| async move {
            match body.decode::<D, S>().await {
                Ok(ret) => Ok(ret),
                Err(err) => {
                    tracing::debug!("request decode body error: {}", err);
                    Err(Error::new(BodyDeserializeError { cause: err.into() }))
                }
            }
        })
        .err_into()
}

#[cfg(feature = "json")]
pub fn json<T: DeserializeOwned + Send, B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, (T,)), Error, Request<B>>,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    decode::<crate::encoder::Json, T, B>()
}

pub fn form<T: DeserializeOwned + Send, B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, (T,)), Error, Request<B>>,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    decode::<crate::encoder::Form, T, B>()
}

// Require the `content-type` header to be this type (or, if there's no `content-type`
// header at all, optimistically hope it's the right type).
fn is_content_type<D: Decoder, B: Body + Send>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
> + Copy {
    |req: Request<B>| async move {
        match _is_content_type::<D, B>(&req) {
            Ok(_) => Outcome::Success((req, ())),
            Err(err) => Outcome::Failure(err),
        }
    }
}

pub fn _is_content_type<D: Decoder, B>(req: &Request<B>) -> Result<(), Error> {
    let (type_, subtype) = D::MIME;
    if let Some(value) = req.headers().get(http::header::CONTENT_TYPE) {
        tracing::trace!("is_content_type {}/{}? {:?}", type_, subtype, value);
        let ct = value
            .to_str()
            .ok()
            .and_then(|s| s.parse::<mime::Mime>().ok());
        if let Some(ct) = ct {
            if ct.type_() == type_ && ct.subtype() == subtype {
                Ok(())
            } else {
                tracing::debug!(
                    "content-type {:?} doesn't match {}/{}",
                    value,
                    type_,
                    subtype
                );
                Err(KnownError::UnsupportMediaType.into())
            }
        } else {
            tracing::debug!("content-type {:?} couldn't be parsed", value);
            Err(KnownError::UnsupportMediaType.into())
        }
    } else if D::WITH_NO_CONTENT_TYPE {
        // Optimistically assume its correct!
        tracing::trace!("no content-type header, assuming {}/{}", type_, subtype);
        Ok(())
    } else {
        tracing::debug!("no content-type found");
        Err(KnownError::UnsupportMediaType.into())
    }
}

#[cfg(feature = "json")]
pub fn any_body<T: DeserializeOwned + Send + 'static, B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, (T,)), Error, Request<B>>,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send + Sync,
{
    json::<T, B>().or(form::<T, B>()).unify()
}

#[derive(Debug)]
pub struct BodyDeserializeError {
    cause: Box<dyn StdError + Send + Sync>,
}

impl fmt::Display for BodyDeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Request body deserialize error: {}", self.cause)
    }
}

impl StdError for BodyDeserializeError {}

#[cfg(feature = "qs")]
pub fn qs<S: DeserializeOwned + Send + 'static, B: Send + 'static>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, (Option<S>,)), Error, Request<B>>,
> + Copy {
    |req: Request<B>| async move {
        match serde_qs::from_str(req.uri().query().unwrap_or("")) {
            Ok(s) => Outcome::Success((req, (Some(s),))),
            Err(e) => unimplemented!("qs fail: {}", e),
        }
    }
}
