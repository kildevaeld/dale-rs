use crate::Outcome;

pub trait IntoResult: Sized {
    type Err;
    type Ok;

    fn into_result(self) -> Result<Self::Ok, Self::Err>;
}

impl<O, E> IntoResult for Result<O, E> {
    type Err = E;
    type Ok = O;
    fn into_result(self) -> Result<O, E> {
        self
    }
}

impl<S, E, N> IntoResult for Outcome<S, E, N> {
    type Err = E;
    type Ok = Option<S>;
    fn into_result(self) -> Result<Self::Ok, Self::Err> {
        self.result()
    }
}
