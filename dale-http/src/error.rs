use crate::filters::BodyReadError;
use dale::Either;
use std::{convert::Infallible, error::Error as StdError, fmt};

pub type Result<T> = std::result::Result<T, Error>;

pub type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
pub struct Error {
    error: BoxError,
}

impl Error {
    pub fn new<E>(error: E) -> Error
    where
        E: Into<BoxError>,
    {
        Error {
            error: error.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dale error: {}", self.error)
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        Some(&*self.error)
    }
}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Error {
        Error {
            error: Box::new(KnownError::Internal(Box::new(error))),
        }
    }
}

impl From<KnownError> for Error {
    fn from(error: KnownError) -> Error {
        Error {
            error: Box::new(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error {
            error: Box::new(error),
        }
    }
}

#[cfg(feature = "hyper")]
impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error {
            error: Box::new(KnownError::Internal(Box::new(error))),
        }
    }
}

impl<E: StdError + Send + Sync + 'static> From<BodyReadError<E>> for Error {
    fn from(error: BodyReadError<E>) -> Error {
        Error {
            error: Box::new(KnownError::Internal(Box::new(error))),
        }
    }
}

impl<L, R> From<Either<L, R>> for Error
where
    L: Into<Error>,
    R: Into<Error>,
{
    fn from(err: Either<L, R>) -> Self {
        match err {
            Either::Left(left) => left.into(),
            Either::Right(right) => right.into(),
        }
    }
}

#[derive(Debug)]
pub enum KnownError {
    Internal(BoxError),
    PayloadTooLarge,
    UnsupportMediaType,
    InvalidHeader(String),
    MissingHeader(String),
    Utf8(std::str::Utf8Error),
}

impl fmt::Display for KnownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KnownError::Internal(err) => write!(f, "internal server error: {}", err),
            KnownError::InvalidHeader(h) => write!(f, "invalid header: {}", h),
            KnownError::MissingHeader(h) => write!(f, "missing header: {}", h),
            KnownError::PayloadTooLarge => write!(f, "payload too large"),
            KnownError::UnsupportMediaType => write!(f, "unsupported media type"),
            KnownError::Utf8(err) => write!(f, "encoding error: {}", err),
        }
    }
}

impl StdError for KnownError {}
