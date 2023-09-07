use crate::{filters::HList, IntoOutcome, Outcome};
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::convert::Infallible;
use either::Either;
#[cfg(feature = "std")]
use std::collections::{BTreeMap, HashMap};

impl<S, E, N> IntoOutcome<N> for Outcome<S, E, N> {
    type Success = S;
    type Failure = E;

    fn into_outcome(self) -> Outcome<S, E, N> {
        self
    }
}

impl<N, T> IntoOutcome<N> for (N, T)
where
    T: HList,
{
    type Success = T::Tuple;

    type Failure = Infallible;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self.1.flatten())
    }
}

impl<S, E, N> IntoOutcome<N> for Result<S, E> {
    type Success = S;
    type Failure = E;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        match self {
            Ok(ret) => Outcome::Success(ret),
            Err(err) => Outcome::Failure(err),
        }
    }
}

impl<S: IntoOutcome<N>, N> IntoOutcome<N> for Option<S> {
    type Success = Option<S::Success>;
    type Failure = S::Failure;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        match self {
            Some(ret) => ret.into_outcome().map(Some).next_then(|_| None),
            None => Outcome::Success(None),
        }
    }
}

impl<L, R, T> IntoOutcome<T> for Either<L, R>
where
    L: IntoOutcome<T>,
    R: IntoOutcome<T>,
{
    type Success = Either<L::Success, R::Success>;

    type Failure = Either<L::Failure, R::Failure>;

    fn into_outcome(self) -> crate::outcome::Outcome<Self::Success, Self::Failure, T> {
        match self {
            Either::Left(left) => left.into_outcome().map(Either::Left).map_err(Either::Left),
            Either::Right(right) => right
                .into_outcome()
                .map(Either::Right)
                .map_err(Either::Right),
        }
    }
}

impl<'a, N> IntoOutcome<N> for &'a str {
    type Failure = Infallible;
    type Success = &'a str;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self)
    }
}

impl<'a, N, T> IntoOutcome<N> for &'a [T] {
    type Failure = Infallible;
    type Success = &'a [T];

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self)
    }
}

macro_rules! outcome {
    ($($ty: ident),*) => {
        $(
            impl<N> IntoOutcome<N> for $ty {
                type Failure = Infallible;
                type Success = $ty;

                fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
                    Outcome::Success(self)
                }
            }
        )*
    };
}

outcome!(u8, i8, u16, i16, u32, i32, u64, i64, isize, usize, f32, f64, bool);

#[cfg(feature = "alloc")]
outcome!(String);

impl<N> IntoOutcome<N> for () {
    type Failure = Infallible;
    type Success = ();

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self)
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<N, T> IntoOutcome<N> for Vec<T> {
    type Failure = Infallible;
    type Success = Vec<T>;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self)
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<N, K, V> IntoOutcome<N> for BTreeMap<K, V> {
    type Failure = Infallible;
    type Success = BTreeMap<K, V>;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self)
    }
}

#[cfg(feature = "std")]
impl<N, K, V> IntoOutcome<N> for HashMap<K, V> {
    type Failure = Infallible;
    type Success = HashMap<K, V>;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        Outcome::Success(self)
    }
}

#[cfg(feature = "http")]
mod _http {
    use super::{Infallible, IntoOutcome, Outcome};
    use http::{Response, StatusCode};

    impl<N, T> IntoOutcome<N> for Response<T> {
        type Failure = Infallible;
        type Success = Response<T>;

        fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
            Outcome::Success(self)
        }
    }

    outcome!(StatusCode);
}
