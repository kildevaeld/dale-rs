use super::Encoder;
use crate::{
    modifier::{Set, With},
    types::Reply,
    Body, Response,
};
use dale::Outcome;
use std::marker::PhantomData;

use dale::IntoOutcome;
use headers::ContentType;
use http::{Request, StatusCode};
use serde::Serialize;

pub fn encode<E: Encoder, S: Serialize>(data: S) -> Encoded<S, E> {
    Encoded(data, false, PhantomData)
}

pub struct Encoded<S, E>(pub(crate) S, pub(crate) bool, pub(crate) PhantomData<E>);

impl<S, E> Encoded<S, E> {
    pub fn pretty(mut self) -> Encoded<S, E> {
        self.1 = true;
        self
    }
}

impl<S, E, B> Reply<B> for Encoded<S, E>
where
    S: Serialize,
    E: Encoder,
    B: Body,
{
    fn into_response(self) -> Response<B> {
        let ret = if self.1 {
            E::encode_pretty(&self.0)
        } else {
            E::encode(&self.0)
        };

        let ret = match ret {
            Ok(s) => s,
            Err(_) => {
                panic!("could not serialize");
            }
        };

        Response::with(StatusCode::OK)
            .set(ret)
            .set(ContentType::from(E::MIME))
    }
}

impl<S, E, B> IntoOutcome<Request<B>> for Encoded<S, E>
where
    S: Serialize,
    E: Encoder,
    B: Body,
{
    type Success = Response<B>;

    type Failure = E::Error;

    fn into_outcome(self) -> dale::Outcome<Self::Success, Self::Failure, Request<B>> {
        let ret = if self.1 {
            E::encode_pretty(&self.0)
        } else {
            E::encode(&self.0)
        };

        match ret {
            Ok(ret) => Outcome::Success(
                Response::with(StatusCode::OK)
                    .set(ret)
                    .set(ContentType::from(E::MIME)),
            ),
            Err(err) => Outcome::Failure(err),
        }
    }
}

#[cfg(feature = "json")]
pub fn json<S: Serialize>(value: S) -> Encoded<S, super::Json> {
    encode(value)
}
