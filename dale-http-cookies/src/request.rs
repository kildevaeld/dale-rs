use dale_http::Request;

use crate::CookieJar;

mod internal {
    use dale_http::Request;

    pub trait Sealed {}

    impl<B> Sealed for Request<B> {}
}

pub trait RequestCookieExt: internal::Sealed {
    fn cookie_jar(&self) -> &CookieJar;
}

impl<B> RequestCookieExt for Request<B> {
    fn cookie_jar(&self) -> &CookieJar {
        self.extensions().get().expect("cookie middleware loaded")
    }
}
