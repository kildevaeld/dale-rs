use dale_http::Request;

mod sealed {
    use dale_http::Request;

    pub trait Sealed {}

    impl<B> Sealed for Request<B> {}
}

pub trait RequestNegotiateExt: sealed::Sealed {
    fn accept(&self) -> Accept;
}

impl<B> RequestNegotiateExt for Request<B> {
    fn accept(&self) -> Accept {
        Accept::new(self)
    }
}

pub struct Accept {}

impl Accept {
    pub fn new<B>(req: &dale_http::Request<B>) -> Accept {
        let accept_hdr = req.headers().get(dale_http::http::header::ACCEPT);
        println!("{:#?}", accept_hdr);
        Accept {}
    }
}
