use cookie::Cookie;
use dale_http::{prelude::*, reply, Request};
use dale_http_cookies::{CookieJar, Cookies, RequestCookieExt};
use hyper::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = |req: Request<hyper::Body>| {
        async move {
            //
            let jar = req.cookie_jar();

            if jar.contains("Hello") {
                let mut cookie = Cookie::named("Hello");
                cookie.make_removal();
                jar.add(cookie);
            } else {
                jar.add(Cookie::new("Hello", "cookie"));
            }

            "Hello, World"
        }
    };

    let service = dale_http::hyper::make(service.wrap(Cookies));

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
