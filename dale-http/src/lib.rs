mod body;
pub mod common;
#[cfg(feature = "serde")]
pub mod encoder;
pub mod error;
pub mod filters;
#[cfg(feature = "fs")]
pub mod fs;
mod modifier;
mod modifiers;
pub mod mount;
pub mod reply_impl;
mod request_ext;
#[cfg(feature = "router")]
pub mod router;
mod types;

#[cfg(feature = "hyper")]
pub mod hyper;

pub use bytes::{self, Bytes};
pub use http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode, Uri};
pub use mime;

pub use self::{
    body::Body,
    error::{Error, KnownError, Result},
    mount::{mount, Mount},
    types::Reply,
};

pub type Outcome<B> = dale::Outcome<Response<B>, error::Error, Request<B>>;

pub mod prelude {
    pub use super::{body::BodyExt, modifier::*, request_ext::*};
    pub use dale::{IntoOutcomeExt, ServiceExt};
}

pub mod reply {
    #[cfg(feature = "serde")]
    pub use super::encoder::reply::*;
    pub use super::reply_impl::*;
}

#[cfg(feature = "headers")]
pub use headers;
