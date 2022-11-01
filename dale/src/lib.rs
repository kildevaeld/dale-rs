#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

mod macros;

#[cfg(feature = "alloc")]
mod impls;
mod into_outcome;
mod middleware;
mod outcome;
mod service;
mod service_ext;

mod types;

pub mod combinators;
pub mod filters;

#[cfg(feature = "alloc")]
pub mod boxed;

pub use self::{into_outcome::*, middleware::*, outcome::*, service::*, service_ext::*};

#[cfg(feature = "alloc")]
pub use self::boxed::BoxService;

mod outcome_impl;

pub use either::Either;

#[cfg(feature = "derive")]
pub use dale_derive::*;

#[cfg(feature = "alloc")]
pub use impls::*;
