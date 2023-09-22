use core::future::Future;
use dale::{boxed::BoxFuture, fail, filters::One, next, success, Outcome, Service, ServiceExt};
use dale_http::{Body, Request, RequestExt};

use crate::{
    filters::{data, id, query},
    method::RestMethod,
    model::{Model, Query},
    route::RestRoute,
    Data, RestRouter,
};

pub struct List<M>
where
    M: Model,
{
    model: M,
    default_query: Option<M::Query>,
}

impl<M: Model> List<M> {
    pub fn new(model: M) -> List<M> {
        List {
            model,
            default_query: None,
        }
    }
}

impl<M, B> Service<Request<B>> for List<M>
where
    B: Send + 'static,
    M: Model + 'static + Send + Clone,
    M::Query: Send + Clone,
    M::Error: std::error::Error + Send + Sync + 'static,
    <M::Query as Query>::Error: Send + Sync + 'static,
{
    type Output = Outcome<(Request<B>, One<Vec<M::Output>>), dale_http::Error, Request<B>>;

    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, req: Request<B>) -> Self::Future {
        let model = self.model.clone();
        let default = self.default_query.clone();
        Box::pin(async move {
            let query = fail!(
                M::Query::from_request(&req, default.as_ref()).map_err(dale_http::Error::new)
            );
            let future = model.list(&query);
            let ret = fail!(future.await.map_err(dale_http::Error::new));
            success!((req, (ret,)))
        })
    }
}

pub struct Create<M>
where
    M: Model,
{
    model: M,
}

impl<M: Model> Create<M> {
    pub fn new(model: M) -> Create<M> {
        Create { model }
    }
}

impl<M, B> Service<Request<B>> for Create<M>
where
    B: Send + dale_http::Body + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
    M: Model + 'static + Send + Clone,
    M::Error: std::error::Error + Send + Sync + 'static,
    M::Data: Send,
    <M::Data as Data>::Error: Send + Sync + 'static,
{
    type Output = Outcome<(Request<B>, One<M::Output>), dale_http::Error, Request<B>>;

    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, mut req: Request<B>) -> Self::Future {
        let model = self.model.clone();
        Box::pin(async move {
            let data = fail!(M::Data::from_request(&mut req)
                .await
                .map_err(dale_http::Error::new));

            let future = model.create(data);
            let ret = fail!(future.await.map_err(dale_http::Error::new));
            success!((req, (ret,)))
        })
    }
}

pub struct Retrieve<M>
where
    M: Model,
{
    model: M,
    key: String,
}

impl<M: Model> Retrieve<M> {
    pub fn new(model: M) -> Retrieve<M> {
        Retrieve {
            model,
            key: String::from("id"),
        }
    }
}

impl<M, B> Service<Request<B>> for Retrieve<M>
where
    B: Send + 'static,
    M: Model + 'static + Send + Clone,
    M::Query: Send + Clone,
    M::Error: std::error::Error + Send + Sync + 'static,
    <M::Query as Query>::Error: Send + Sync + 'static,
{
    type Output = Outcome<(Request<B>, One<M::Output>), dale_http::Error, Request<B>>;

    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, req: Request<B>) -> Self::Future {
        let model = self.model.clone();
        let key = self.key.clone();
        Box::pin(async move {
            let id = match req.params().get(&key) {
                Some(id) => id,
                None => return next!(req),
            };

            let future = model.get(id);
            let ret = fail!(future.await.map_err(dale_http::Error::new));
            success!((req, (ret,)))
        })
    }
}

pub fn list<M: Model>(name: &str, model: M) -> RestRoute<String, List<M>> {
    RestRoute {
        segments: format!("/{name}"),
        service: List {
            model: model,
            default_query: None,
        },
        method: RestMethod::List,
    }
}

pub fn retrieve<M: Model>(name: &str, model: M) -> RestRoute<String, Retrieve<M>> {
    RestRoute {
        segments: format!("/{name}/:{name}-id"),
        service: Retrieve {
            model,
            key: format!("{name}-id"),
        },
        method: RestMethod::Retrieve,
    }
}

pub fn create<M: Model>(name: &str, model: M) -> RestRoute<String, Create<M>> {
    RestRoute {
        segments: format!("/{name}"),
        service: Create { model },
        method: RestMethod::Create,
    }
}

pub struct RouteSet<M: Model> {
    model: M,
    name: String,
    create: Option<Create<M>>,
    retrieve: Option<Retrieve<M>>,
    list: Option<List<M>>,
}

impl<M: Model + Clone> RouteSet<M> {
    pub fn new(name: &str, model: M) -> RouteSet<M> {
        RouteSet {
            model,
            name: name.to_string(),
            create: None,
            retrieve: None,
            list: None,
        }
    }
    pub fn register() {}

    pub fn create(mut self) -> Self {
        self.create = Some(Create {
            model: self.model.clone(),
        });
        self
    }

    pub fn list(mut self) -> Self {
        self.list = Some(List {
            model: self.model.clone(),
            default_query: None,
        });
        self
    }

    pub fn all(self) -> Self {
        self.list().create()
    }
}

impl<M: Model> RouteSet<M> {
    pub fn attach<B>(self, router: &mut RestRouter<B>)
    where
        B: Body + 'static + Send,
        B::Error: std::error::Error + Send + Sync + 'static,
        M: Model + 'static + Send + Sync + Clone,
        M::Output: serde::ser::Serialize + Send,
        M::Query: Send + Sync + Clone,
        M::Error: std::error::Error + Send + Sync + 'static,
        <M::Query as Query>::Error: Send + Sync + 'static,
        M::Data: Send,
        <M::Data as Data>::Error: Send + Sync + 'static,
    {
        if let Some(list) = self.list {
            router
                .register(RestRoute::list(&self.name, list).unwrap())
                .unwrap();
        }

        if let Some(create) = self.create {
            router
                .register(RestRoute::create(&self.name, create).unwrap())
                .unwrap();
        }

        if let Some(retrieve) = self.retrieve {
            router
                .register(RestRoute::retrieve(&self.name, retrieve).unwrap())
                .unwrap();
        }
    }
}
