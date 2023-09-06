use async_trait::async_trait;

use crate::session::SessionId;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    data: odu_value::Map,
}

impl SessionData {
    pub fn get<T: serde::Deserialize>(&self, key: &str) -> Result<T, ()> {}
}

#[async_trait]
pub trait Store {
    async fn load(&self, session_id: &SessionId) -> Result<SessionData, ()>;
    async fn save(&self, session_id: &SessionId, data: &SessionData) -> Result<(), ()>;
}
