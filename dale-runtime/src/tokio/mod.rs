mod executor;
#[cfg(feature = "fs")]
mod fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tokio;
