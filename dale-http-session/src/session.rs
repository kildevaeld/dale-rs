use std::sync::Arc;

use dale_http::Request;
use uuid::Uuid;

use crate::{manager::Manager, store::SessionData};

const SESSION_KEY: &str = "sess_id";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

// impl<'a> From<&'a str> for SessionId {
//     fn from(value: &'a str) -> Self {
//         SessionId(Arc::from(value))
//     }
// }

pub trait SessionIdExtractor<B> {
    fn extract(&self, req: &Request<B>) -> Option<SessionId>;
}

pub struct Session<B> {
    pub(crate) id: SessionId,
    pub(crate) manager: Manager<B>,
    pub(crate) data: SessionData,
    pub(crate) modified: bool,
}

impl<B> Session<B> {
    pub async fn save(&mut self) {}

    pub async fn load(&mut self) {
        self.manager.load(&self.id).await
    }

    pub fn get<T: serde::Deserialize>(&self, key: &str) -> T {}
}
