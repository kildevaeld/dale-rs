use std::sync::Arc;

use dale_http::Request;

use crate::manager::Manager;

const SESSION_KEY: &str = "sess_id";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId(Arc<str>);

impl<'a> From<&'a str> for SessionId {
    fn from(value: &'a str) -> Self {
        SessionId(Arc::from(value))
    }
}

pub trait SessionIdExtractor<B> {
    fn extract(&self, req: &Request<B>) -> Option<SessionId>;
}

pub struct Session<B> {
    pub(crate) id: SessionId,
    pub(crate) manager: Manager<B>,
}

impl<B> Session<B> {
    pub async fn save(&mut self) {}

    pub async fn load(&mut self) {
        self.manager.load(&self.id).await
    }
}
