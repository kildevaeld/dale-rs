use dale_http::{router::Router, Request, RequestExt};
use hyper::{Body, Server};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let mut router = Router::new();

    router
        .get("/", |_: Request<_>| async move { "Hello, World!" })?
        .get("/upper/:name", |req: Request<Body>| async move {
            let params = req.params().get("name").unwrap();

            params.to_uppercase()
        })?;

    let service = dale_http::hyper::make(router.into_service());

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
