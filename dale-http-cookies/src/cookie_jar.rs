use dale_http::{
    prelude::{Modifier, Set},
    Body, HeaderValue, Response,
};
use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CookieJar(pub(crate) Arc<Mutex<cookie::CookieJar>>);

impl CookieJar {
    pub fn add(&self, cookie: cookie::Cookie<'static>) {
        self.lock().add(cookie)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.lock().get(name).is_some()
    }

    pub fn lock(&self) -> CookieJarLock<'_> {
        CookieJarLock(self.0.lock())
    }
}

pub struct CookieJarLock<'a>(MutexGuard<'a, cookie::CookieJar>);

impl<'a> std::ops::Deref for CookieJarLock<'a> {
    type Target = cookie::CookieJar;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> CookieJarLock<'a> {
    pub fn get(&self, name: &str) -> Option<&cookie::Cookie<'static>> {
        self.0.get(name)
    }

    pub fn add(&mut self, cookie: cookie::Cookie<'static>) {
        self.0.add(cookie);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.get(name).is_some()
    }
}

impl<B> Modifier<Response<B>> for CookieJar {
    fn modify(self, resp: &mut Response<B>) {
        resp.set_mut(&self);
    }
}

impl<'a, B> Modifier<Response<B>> for &'a CookieJar {
    fn modify(self, resp: &mut Response<B>) {
        let lock = self.lock();

        for cookie in lock.delta() {
            resp.headers_mut().insert(
                dale_http::http::header::SET_COOKIE,
                HeaderValue::from_str(&cookie.to_string()).unwrap(),
            );
        }
    }
}
