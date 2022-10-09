mod body;
pub mod common;
#[cfg(feature = "serde")]
mod encoder;
pub mod error;
pub mod filters;
#[cfg(feature = "fs")]
pub mod fs;
mod modifier;
mod modifiers;
pub mod mount;
pub mod reply_impl;
mod types;

mod request_ext;

#[cfg(feature = "hyper")]
pub mod hyper;

pub use http::{Method, Request, Response, StatusCode, Uri};

pub use self::{
    body::Body,
    error::Result,
    mount::{mount, Mount},
    types::Reply,
};

pub type Outcome<B> = dale::Outcome<Response<B>, error::Error, Request<B>>;

pub mod prelude {
    pub use super::{modifier::*, request_ext::*};
    pub use dale::{IntoOutcomeExt, ServiceExt};
}

pub mod reply {
    #[cfg(feature = "serde")]
    pub use super::encoder::reply::*;
    pub use super::reply_impl::*;
}

//
mod taker;
pub use self::taker::*;
