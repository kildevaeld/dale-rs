use dale_http::{filters, reply, Result};
use dale_http::{prelude::*, Request};
use hyper::{Body, Server};

use dale_http_negotiate::RequestNegotiateExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = |req: Request<_>| async move {
        let accept = req.accept();
        "Hello"
    };

    let service = dale_http::hyper::make(service);

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
