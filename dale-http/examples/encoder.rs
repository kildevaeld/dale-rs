use dale_http::{prelude::*, Request};
use hyper::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service =
        |req: Request<hyper::Body>| async move { dale_http::reply::json("Hello, World!") };

    let service = dale_http::hyper::make(service);

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
