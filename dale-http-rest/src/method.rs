use dale_http::Method;

pub enum RestMethod {
    Create,
    Update,
    Patch,
    List,
    Retrieve,
    Delete,
}

impl From<RestMethod> for Method {
    fn from(value: RestMethod) -> Self {
        match value {
            RestMethod::Create => Method::POST,
            RestMethod::List | RestMethod::Retrieve => Method::GET,
            RestMethod::Update => Method::PUT,
            RestMethod::Patch => Method::PATCH,
            RestMethod::Delete => Method::DELETE,
        }
    }
}
