use dale::{Either, IntoOutcome, Middleware, Outcome, Service};
use futures_core::ready;
use http::{Request, Uri};
use pin_project_lite::pin_project;
use std::{
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

pub fn mount<S: ToString, T, B>(path: S, task: T) -> MountTask<T, B>
where
    T: Service<Request<B>> + Clone,
{
    Mount::new(path).wrap(task)
}

fn normalize_path(mut path: String) -> String {
    if path.is_empty() {
        path.push('/');
    }
    if path.chars().nth(0).unwrap() != '/' {
        path.insert(0, '/');
    }
    let len = path.len();
    if path.chars().nth(len - 1).unwrap() != '/' {
        path.push('/');
    }
    path
}

pub struct Mount(String);

impl Mount {
    pub fn new<S: ToString>(path: S) -> Mount {
        let path = normalize_path(path.to_string());
        Mount(path)
    }
}

impl<B, T> Middleware<Request<B>, T> for Mount
where
    T: Service<Request<B>> + Clone,
{
    type Service = MountTask<T, B>;
    fn wrap(&self, service: T) -> Self::Service {
        MountTask {
            path: self.0.clone(),
            task: service,
            _b: PhantomData,
        }
    }
}

pub struct MountTask<T, B> {
    pub(crate) path: String,
    task: T,
    _b: PhantomData<B>,
}

impl<T, B> Clone for MountTask<T, B>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        MountTask {
            path: self.path.clone(),
            task: self.task.clone(),
            _b: PhantomData,
        }
    }
}

unsafe impl<T: Send, B> Send for MountTask<T, B> {}

unsafe impl<T: Sync, B> Sync for MountTask<T, B> {}

impl<T, B> MountTask<T, B> {
    #[inline]
    fn starts_with(&self, url: &Uri) -> bool {
        let path = url.path();
        if path.len() < self.path.len() {
            path.starts_with(&self.path.as_str()[0..(self.path.len() - 1)])
        } else {
            path.starts_with(self.path.as_str())
        }
    }

    #[inline]
    fn replace_path(&self, url: &Uri) -> Uri {
        let p = {
            let path = url.path();
            let path = if path.ends_with("/") {
                &path[self.path.len()..]
            } else {
                &path[(self.path.len() - 1)..]
            };
            if path.is_empty() {
                "/"
            } else {
                path
            }
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
            o.extend(&[":", p.as_str()]);
        }

        o.push(&p);
        if let Some(s) = url.query() {
            o.extend(&["?", s]);
        }

        Uri::from_str(&o.join("")).unwrap()
    }

    #[inline]
    fn ensure_mount(&self, req: &mut Request<B>, path: String) {
        if req.extensions().get::<MountPath>().is_none() {
            req.extensions_mut().insert(MountPath(Vec::default()));
        }
        req.extensions_mut()
            .get_mut::<MountPath>()
            .unwrap()
            .0
            .push(path);
    }
}

impl<T, B> Service<Request<B>> for MountTask<T, B>
where
    T: Service<Request<B>>,
{
    type Output = Outcome<
        <T::Output as IntoOutcome<Request<B>>>::Success,
        <T::Output as IntoOutcome<Request<B>>>::Failure,
        Request<B>,
    >;

    type Future = Either<MountFuture<T::Future, B>, std::future::Ready<Self::Output>>;

    #[inline(always)]
    fn call(&self, mut req: Request<B>) -> Self::Future {
        if self.starts_with(req.uri()) {
            let url = req.uri().clone();
            *req.uri_mut() = self.replace_path(req.uri());
            self.ensure_mount(&mut req, self.path[0..self.path.len() - 1].to_string());
            Either::Left(MountFuture::new(self.task.call(req), url))
        } else {
            Either::Right(std::future::ready(Outcome::Next(req)))
        }
    }
}

pin_project! {
    pub struct MountFuture<T, B>
    {
        #[pin]
        future: T,
        uri: Option<Uri>,
        _b: PhantomData<B>
    }
}

impl<T, B> MountFuture<T, B> {
    fn new(future: T, uri: Uri) -> MountFuture<T, B> {
        MountFuture {
            future,
            uri: uri.into(),
            _b: PhantomData,
        }
    }
}

impl<T, B> Future for MountFuture<T, B>
where
    T: Future,
    T::Output: IntoOutcome<Request<B>>,
{
    type Output = Outcome<
        <T::Output as IntoOutcome<Request<B>>>::Success,
        <T::Output as IntoOutcome<Request<B>>>::Failure,
        Request<B>,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match ready!(this.future.poll(cx)).into_outcome() {
            Outcome::Next(mut req) => {
                *req.uri_mut() = this.uri.take().unwrap();
                Poll::Ready(Outcome::Next(req))
            }
            o => Poll::Ready(o),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MountPath(Vec<String>);

impl MountPath {
    pub fn push<S: ToString, B>(req: &mut Request<B>, path: S) {
        if req.extensions().get::<MountPath>().is_none() {
            req.extensions_mut().insert(MountPath(Vec::default()));
        }
        req.extensions_mut()
            .get_mut::<MountPath>()
            .unwrap()
            .0
            .push(path.to_string());
    }

    pub fn real_path<B>(&self, req: &Request<B>) -> String {
        let mut out = self.0.join("");
        out.push_str(req.uri().path());
        out
    }
}

impl fmt::Display for MountPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("/"))
    }
}
