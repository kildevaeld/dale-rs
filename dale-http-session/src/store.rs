use async_trait::async_trait;

use crate::session::SessionId;

#[async_trait]
pub trait Store {
    async fn load(&self, session_id: &SessionId);
}
