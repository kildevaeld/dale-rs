use std::str::FromStr;

use dale::{filters::One, Outcome, Service};
use http::{Request, Uri};

use crate::{error::Error, mount::MountPath};

#[derive(Debug, Clone, Copy)]
pub struct RealPath;

impl<B> Service<Request<B>> for RealPath {
    type Output = Outcome<(Request<B>, One<String>), Error, Request<B>>;

    type Future = std::future::Ready<Self::Output>;

    fn call(&self, req: Request<B>) -> Self::Future {
        let m = match req.extensions().get::<MountPath>() {
            Some(p) => p.real_path(&req),
            None => req.uri().path().to_owned(),
        };
        std::future::ready(Outcome::Success((req, (m,))))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RealUri;

impl<B> Service<Request<B>> for RealUri {
    type Output = Outcome<(Request<B>, One<Uri>), Error, Request<B>>;

    type Future = std::future::Ready<Self::Output>;

    fn call(&self, req: Request<B>) -> Self::Future {
        let url = req.uri();
        let p = match req.extensions().get::<MountPath>() {
            Some(p) => p.real_path(&req),
            None => req.uri().path().to_owned(),
        };
        let port = url.port();
        let mut o = Vec::default();
        if let Some(s) = url.scheme_str() {
            o.push(s);
        }
        if let Some(s) = url.authority() {
            o.push(s.as_str());
        }
        if let Some(p) = &port {
            o.extend([":", p.as_str()]);
        }

        o.push(&p);
        if let Some(s) = url.query() {
            o.extend(["?", s]);
        }

        let uri = Uri::from_str(&o.join("")).unwrap();
        std::future::ready(Outcome::Success((req, (uri,))))
    }
}

pub fn realuri() -> RealUri {
    RealUri
}

pub fn realpath() -> RealPath {
    RealPath
}
