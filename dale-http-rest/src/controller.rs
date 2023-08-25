use async_trait::async_trait;
use dale_http::Request;

#[async_trait]
pub trait Controller<B> {
    type Output;
    type Error;
    async fn list(&self, req: &mut Request<B>) -> Result<Self::Output, Self::Error>;
    async fn retrieve(&self, req: &mut Request<B>) -> Result<Self::Output, Self::Error>;
    async fn create(&self, req: &mut Request<B>) -> Result<Self::Output, Self::Error>;
    async fn update(&self, req: &mut Request<B>) -> Result<Self::Output, Self::Error>;
}
