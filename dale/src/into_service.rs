use super::Service;

pub trait IntoService<R> {
    type Error;
    type Service: Service<R>;
    fn into_service(self) -> Result<Self::Service, Self::Error>;
}
