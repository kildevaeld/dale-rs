use dale_http::{filters, reply, Result};
use dale_http::{prelude::*, Request};
use hyper::{Body, Server};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = filters::get()
        .and(filters::path("/"))
        .map(|| "Hello, World!")
        .or(filters::get()
            .and(filters::path("/test"))
            .map(reply::html("<h1>Hello, World!</h1>")))
        // .or(|mut req: Request<Body>| async move {
        //     let bytes = req.text().await?;
        //     Result::Ok(format!("Hello: {:?}", bytes))
        // })
        .or(filters::post()
            .or(filters::put())
            .unify()
            .and(filters::text())
            .map(|body| reply::text(format!("Hello: {}", body))))
        // .or(filters::method().and_then(|_method| async move {
        //     //
        //     Result::Ok("And then this")
        // }))
        .or(reply::text("Wildcard"));

    let service = dale_http::hyper::make(service);

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
