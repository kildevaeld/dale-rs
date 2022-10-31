use dale_http::prelude::*;
use hyper::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = dale_http::hyper::make(dale::filters::any().map(|| "Hello, World!"));

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
