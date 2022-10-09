use dale::{IntoOutcome, Service};

#[derive(IntoOutcome)]
pub struct Test<A> {
    value: A,
}

fn main() {
    let service = dale::service(|args: u32| async move { Test { value: args } });

    service.call(42);
}
