use std::sync::Arc;

use dale_http::Request;
use keyval::{Cbor, KeyVal, Store, TtlStore};

use crate::{
    session::{Session, SessionId, SessionIdExtractor},
    store::SessionData,
};

struct ManagerInner<B> {
    store: KeyVal<Box<dyn TtlStore>>,
    extractor: Box<dyn SessionIdExtractor<B> + Send + Sync>,
}

pub struct Manager<B>(Arc<ManagerInner<B>>);

impl<B> Clone for Manager<B> {
    fn clone(&self) -> Self {
        Manager(self.0.clone())
    }
}

impl<B> Manager<B> {
    pub fn session(&self, req: &Request<B>) -> Session<B> {
        let id = if let Some(id) = self.0.extractor.extract(req) {
            id
        } else {
            "test_id".into()
        };

        Session {
            id,
            manager: self.clone(),
        }
    }

    pub(crate) async fn load(&self, session: &SessionId) {
        let result: Cbor<SessionData> = self.0.store.get(session.as_bytes()).await.unwrap();
    }
}
