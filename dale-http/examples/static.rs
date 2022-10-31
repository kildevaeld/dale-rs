use std::path::Path;

use dale::filters::any;
use dale_http::prelude::*;
use dale_http::reply;
use hyper::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = dale_http::fs::root(Path::new("."))
        .map(reply::json)
        .wrap(dale_http::Mount::new("/statics"))
        .or(any().map(|| "Hello"));

    let service = dale_http::hyper::make(service);

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
