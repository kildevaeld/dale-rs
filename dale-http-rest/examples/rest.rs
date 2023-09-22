use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use async_trait::async_trait;
use dale::IntoService;
use dale_http::router::Router;
use dale_http::{filters, reply, Body};
use dale_http::{prelude::*, Request};
use hyper::Server;

use dale_http_rest::{Create, Data, List, Model, RestRouter, Retrieve, RouteSet};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Todo {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone, Default)]
struct ModelImpl(Arc<Mutex<HashMap<uuid::Uuid, Todo>>>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTodo {
    name: String,
}

#[async_trait]
impl Data for CreateTodo {
    type Error = dale_http::Error;
    async fn from_request<B: Body + Send>(req: &mut Request<B>) -> Result<Self, Self::Error>
    where
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        req.json::<CreateTodo>()
            .await
            .map_err(dale_http::Error::new)
    }
}

#[async_trait]
impl Model for ModelImpl {
    type Query = ();
    type Data = CreateTodo;
    type Error = dale_http::Error;
    type Output = Todo;

    async fn count(&self, query: &Self::Query) -> Result<u64, Self::Error> {
        Ok(self.0.lock().await.len() as u64)
    }
    async fn list(&self, query: &Self::Query) -> Result<Vec<Self::Output>, Self::Error> {
        Ok(self.0.lock().await.values().cloned().collect::<_>())
    }
    async fn get(&self, id: &str) -> Result<Self::Output, Self::Error> {
        let id = Uuid::parse_str(id).map_err(dale_http::Error::new)?;

        let found = self
            .0
            .lock()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| dale_http::Error::new("not found"))?;

        Ok(found)
    }
    async fn create(&self, data: Self::Data) -> Result<Self::Output, Self::Error> {
        let id = Uuid::new_v4();
        let todo = Todo {
            id,
            name: data.name,
        };

        self.0.lock().await.insert(id, todo.clone());

        Ok(todo)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let model = ModelImpl::default();

    let mut router = RestRouter::default();

    RouteSet::new("todos", model.clone())
        .all()
        .attach(&mut router);

    // router.get("/", List::new(model.clone()).service().map(reply::json))?;

    // router.post("/", Create::new(model.clone()).service().map(reply::json))?;

    // router.get(
    //     "/:id",
    //     Retrieve::new(model.clone()).service().map(reply::json),
    // )?;

    let service = dale_http::hyper::make(router.into_service()?);

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
