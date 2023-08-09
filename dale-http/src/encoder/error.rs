use std::{error::Error, fmt};

use crate::error::BoxError;

#[derive(Debug)]
pub enum DecodeErrorKind {
    UnsupportedMediaType,
    Syntax(BoxError),
    Transport(BoxError),
}

#[derive(Debug)]
pub struct DecodeError {
    pub kind: DecodeErrorKind,
}

impl DecodeError {
    pub fn transport<E>(error: E) -> DecodeError
    where
        E: Error + Send + Sync + 'static,
    {
        DecodeError {
            kind: DecodeErrorKind::Transport(Box::new(error)),
        }
    }

    pub fn syntax<E>(error: E) -> DecodeError
    where
        E: Error + Send + Sync + 'static,
    {
        DecodeError {
            kind: DecodeErrorKind::Syntax(Box::new(error)),
        }
    }

    pub fn unsupported() -> DecodeError {
        DecodeError {
            kind: DecodeErrorKind::UnsupportedMediaType,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DecodeErrorKind::Syntax(err) => write!(f, "syntax error: {}", err),
            DecodeErrorKind::UnsupportedMediaType => write!(f, "unsupported mediatype"),
            DecodeErrorKind::Transport(err) => write!(f, "transport error: {}", err),
        }
    }
}

impl Error for DecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.kind {
            DecodeErrorKind::Syntax(ref err) => Some(err.as_ref()),
            DecodeErrorKind::Transport(ref err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct EncodeError {
    inner: Box<dyn Error + Send + Sync>,
}

impl EncodeError {
    pub fn new<E>(error: E) -> EncodeError
    where
        E: Error + Send + Sync + 'static,
    {
        EncodeError {
            inner: Box::new(error),
        }
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

impl From<serde_urlencoded::de::Error> for DecodeError {
    fn from(value: serde_urlencoded::de::Error) -> Self {
        DecodeError::syntax(value)
    }
}

impl From<serde_urlencoded::ser::Error> for EncodeError {
    fn from(value: serde_urlencoded::ser::Error) -> Self {
        EncodeError::new(value)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for DecodeError {
    fn from(value: serde_json::Error) -> Self {
        DecodeError::syntax(value)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for EncodeError {
    fn from(value: serde_json::Error) -> Self {
        EncodeError::new(value)
    }
}
