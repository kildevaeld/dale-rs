use http::Request;

use crate::{
    common::{ToBytes, ToText},
    Body,
};

mod privatet {
    use http::Request;

    pub trait Sealed {}
    impl<B> Sealed for Request<B> {}
}

pub trait RequestExt<B> {
    fn bytes(&mut self) -> ToBytes<B>
    where
        B: Body;

    fn text<'a>(&'a mut self) -> ToText<B>
    where
        B: Body;
}

impl<B> RequestExt<B> for Request<B> {
    fn bytes(&mut self) -> ToBytes<B>
    where
        B: Body,
    {
        let body = std::mem::replace(self.body_mut(), B::empty());
        ToBytes::new(body)
    }

    fn text<'a>(&mut self) -> ToText<B>
    where
        B: Body,
    {
        let body = std::mem::replace(self.body_mut(), B::empty());

        ToText::new(body)
    }
}
