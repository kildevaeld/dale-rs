use std::path::Path;

use dale::filters::any;
use dale_http::{filters, reply, Result};
use dale_http::{prelude::*, Request};
use hyper::{Body, Server};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = dale_http::hyper::make(dale::filters::any().map(|| "Hello, World!"));

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
