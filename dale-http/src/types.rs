use http::Response;

pub trait Reply<B> {
    fn into_response(self) -> Response<B>;
}
