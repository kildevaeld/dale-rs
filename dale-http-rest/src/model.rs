use std::{convert::Infallible, future::Future};

use async_trait::async_trait;
use dale::{filters::One, IntoOutcome, Outcome, Service, ServiceExt};
use dale_http::{Body, Request, RequestExt};

pub trait Query: Sized {
    type Error: std::error::Error;
    fn from_request<B>(req: &Request<B>, default: Option<&Self>) -> Result<Self, Self::Error>;
}

impl Query for () {
    type Error = Infallible;
    fn from_request<B>(req: &Request<B>, default: Option<&Self>) -> Result<Self, Self::Error> {
        Ok(())
    }
}

#[async_trait]
pub trait Data: Sized {
    type Error: std::error::Error;
    async fn from_request<B: Body + Send>(req: &mut Request<B>) -> Result<Self, Self::Error>
    where
        B::Error: std::error::Error + Send + Sync + 'static;
}

#[async_trait]
pub trait Model {
    type Query: Query;
    type Data: Data;
    type Error;
    type Output;

    async fn count(&self, query: &Self::Query) -> Result<u64, Self::Error>;
    async fn list(&self, query: &Self::Query) -> Result<Vec<Self::Output>, Self::Error>;
    async fn get(&self, id: &str) -> Result<Self::Output, Self::Error>;
    async fn create(&self, data: Self::Data) -> Result<Self::Output, Self::Error>;
}
