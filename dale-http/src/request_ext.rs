use http::Request;
#[cfg(feature = "headers")]
use mime::Mime;

use crate::{
    body::BodyExt,
    common::{ToBytes, ToText},
    Body,
};

#[cfg(feature = "json")]
use crate::encoder::Json;
#[cfg(feature = "serde")]
use crate::encoder::{Decoder, Form, ToDecoded};

mod sealed {
    use http::Request;

    pub trait Sealed {}
    impl<B> Sealed for Request<B> {}
}

pub trait RequestExt<B>: sealed::Sealed {
    #[cfg(feature = "headers")]
    fn content_type(&self) -> Mime;

    #[cfg(feature = "headers")]
    fn content_length(&self) -> Option<u64>;

    #[cfg(feature = "headers")]
    fn host(&self) -> Option<headers::Host>;

    fn bytes(&mut self) -> ToBytes<B>
    where
        B: Body;

    fn text(&mut self) -> ToText<B>
    where
        B: Body;

    #[cfg(feature = "serde")]
    fn decode<D: Decoder, S: serde::de::DeserializeOwned>(&mut self) -> ToDecoded<D, S, B>
    where
        B: Body;

    #[cfg(feature = "json")]
    fn json<S>(&mut self) -> ToDecoded<Json, S, B>
    where
        B: Body,
        S: serde::de::DeserializeOwned,
    {
        self.decode()
    }

    #[cfg(feature = "serde")]
    fn form<S: serde::de::DeserializeOwned>(&mut self) -> ToDecoded<Form, S, B>
    where
        B: Body,
    {
        self.decode()
    }

    #[cfg(feature = "router")]
    fn params(&self) -> &crate::router::Params;
}

impl<B> RequestExt<B> for Request<B> {
    #[cfg(feature = "headers")]
    fn content_type(&self) -> Mime {
        use headers::HeaderMapExt;
        self.headers()
            .typed_get::<headers::ContentType>()
            .map(|m| m.into())
            .unwrap_or_else(|| mime::APPLICATION_OCTET_STREAM)
    }

    #[cfg(feature = "headers")]
    fn content_length(&self) -> Option<u64> {
        use headers::HeaderMapExt;
        self.headers()
            .typed_get::<headers::ContentLength>()
            .map(|m| m.0)
    }

    #[cfg(feature = "headers")]
    fn host(&self) -> Option<headers::Host> {
        use headers::HeaderMapExt;
        self.headers().typed_get::<headers::Host>()
    }

    fn bytes(&mut self) -> ToBytes<B>
    where
        B: Body,
    {
        let body = std::mem::replace(self.body_mut(), B::empty());
        ToBytes::new(body)
    }

    fn text(&mut self) -> ToText<B>
    where
        B: Body,
    {
        let body = std::mem::replace(self.body_mut(), B::empty());
        ToText::new(body)
    }

    #[cfg(feature = "serde")]
    fn decode<D: Decoder, S: serde::de::DeserializeOwned>(&mut self) -> ToDecoded<D, S, B>
    where
        B: Body,
    {
        let body = std::mem::replace(self.body_mut(), B::empty());
        body.decode()
    }

    #[cfg(feature = "router")]
    fn params(&self) -> &crate::router::Params {
        static PARAMS: crate::router::Params = crate::router::Params::new();
        self.extensions().get().unwrap_or(&PARAMS)
    }
}
