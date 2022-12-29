use dale::{BoxService, IntoOutcome, IntoService, Service, ServiceExt};

#[derive(IntoOutcome)]
pub struct Test<A> {
    value: A,
}

pub struct Serv;

impl<R> IntoService<Test<R>> for Serv {
    type Error = core::convert::Infallible;
    type Service = BoxService<'static, Test<R>, i32, core::convert::Infallible>;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        Ok((|_| async move { 42 }).boxed())
    }
}

fn main() {
    let service = dale::service(|args: u32| async move { Test { value: args } });

    service.call(42);
}
