use super::Service;

pub trait IntoService<R> {
    type Error;
    type Service: Service<R>;
    fn into_service(self) -> Result<Self::Service, Self::Error>;
}

impl<S, R> IntoService<R> for S
where
    S: Service<R>,
{
    type Error = core::convert::Infallible;
    type Service = S;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        Ok(self)
    }
}
