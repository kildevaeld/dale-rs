use std::sync::Arc;

use dale_http::Request;

use crate::{
    session::{Session, SessionId, SessionIdExtractor},
    store::Store,
};

struct ManagerInner<B> {
    store: Box<dyn Store>,
    extractor: Box<dyn SessionIdExtractor<B>>,
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
        self.0.store.load(session).await;
    }
}
