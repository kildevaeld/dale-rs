use dale::{Outcome, Service, ServiceExt};
use futures_core::Future;
use http::Request;
use serde::de::DeserializeOwned;
use std::error::Error as StdError;
use std::fmt;

use crate::encoder::{Decoder, Form};
use crate::error::{Error, KnownError};
use crate::{filters, Body};

#[cfg(feature = "json")]
pub fn json<T: DeserializeOwned + Send, B: Body + Send + 'static>(
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<T, Error, Request<B>>> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    is_content_type::<Form, B>()
        .and(filters::aggregate())
        .and_then(|(_, (buf,))| async move {
            crate::encoder::Json::decode::<_, T>(buf).map_err(|err| {
                tracing::debug!("request form body error: {}", err);
                Error::new(BodyDeserializeError { cause: err.into() })
            })
        })
        .err_into()
}
// pub fn to_json<S: serde::Serialize + Send + 'static, B: Default + From<Vec<u8>> + Send>(
// ) -> impl Service<(Request<B>, (S,)), Output = impl Reply<B>, Error = Error> + Copy {
//     |(_, (data,))| async move { Ok(reply::json(data)) }
// }

pub fn form<T: DeserializeOwned + Send, B: Body + Send + 'static>(
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<T, Error, Request<B>>> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    is_content_type::<Form, B>()
        .and(filters::aggregate())
        .and_then(|(_, (buf,))| async move {
            Form::decode::<_, T>(buf).map_err(|err| {
                tracing::debug!("request form body error: {}", err);
                Error::new(BodyDeserializeError { cause: err.into() })
            })
        })
        .err_into()
}

// Require the `content-type` header to be this type (or, if there's no `content-type`
// header at all, optimistically hope it's the right type).
fn is_content_type<D: Decoder, B: Body + Send + 'static>() -> impl Service<
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
pub fn any_body<T: DeserializeOwned + Send + 'static, B: Body + Send + Default + 'static>(
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<T, Error, Request<B>>> + Clone
where
    B::Data: Send,
    B::Error: Into<Error> + Send + Sync,
{
    use dale::Either;

    json::<T, B>()
        .or(form::<T, B>())
        .and_then(|e| async {
            let ret = match e {
                Either::Left(l) => l,
                Either::Right(r) => r,
            };

            crate::Result::Ok(ret)
        })
        .err_into()
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

// pub fn qs<S: DeserializeOwned + Send + 'static, B: Send + 'static>() -> impl Service<
//     Request<B>,
//     Output = (Request<B>, (Option<S>,)),
//     Error = Error,
//     Future = impl Future + Send,
// > + Copy {
//     |req: Request<B>| async move {
//         let m = match serde_qs::from_str(req.uri().query().unwrap_or("")) {
//             Ok(s) => Some(s),
//             Err(e) => unimplemented!("qs fail: {}", e),
//         };

//         Ok((req, (m,)))
//     }
// }
