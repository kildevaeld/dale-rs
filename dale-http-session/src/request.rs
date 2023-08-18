use dale_http::Request;

use crate::{manager::Manager, session::Session};

mod sealed {
    use dale_http::Request;

    pub trait Sealed {}

    impl<B> Sealed for Request<B> {}
}

pub trait RequestSessionExt<B>: sealed::Sealed {
    fn session(&mut self) -> &mut Session<B>;
}

impl<B: 'static> RequestSessionExt<B> for Request<B> {
    fn session(&mut self) -> &mut Session<B> {
        let manager = self.extensions_mut().get_mut::<Manager<B>>();
        todo!()
    }
}
