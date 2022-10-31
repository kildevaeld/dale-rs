use dale::{IntoOutcome, Middleware, Outcome, Service};
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

impl<T, B> Service<Request<B>> for MountTask<T, B>
where
    T: Service<Request<B>> + Clone,
{
    type Output = Outcome<
        <T::Output as IntoOutcome<Request<B>>>::Success,
        <T::Output as IntoOutcome<Request<B>>>::Failure,
        Request<B>,
    >;

    type Future = MountFuture<T, B>;

    #[inline(always)]
    fn call(&self, req: Request<B>) -> Self::Future {
        MountFuture::new(self.task.clone(), self.path.clone(), req)
    }
}

pin_project! {
    pub struct MountFuture<T, B>
    where T: Service<Request<B>>
    {
        #[pin]
        state: State<T, B>
    }
}

pin_project! {
    #[project = StatProj]
    enum State<T, B> where T: Service<Request<B>> {
        Init {
            task: T,
            req: Option<Request<B>>,
            path: String,
        },
        Future {
            #[pin]
            future: T::Future,
            uri: Option<Uri>
        }
    }
}

impl<T, B> MountFuture<T, B>
where
    T: Service<Request<B>>,
{
    fn new(task: T, path: String, req: Request<B>) -> MountFuture<T, B> {
        MountFuture {
            state: State::Init {
                task,
                req: Some(req),
                path,
            },
        }
    }
}

impl<T, B> Future for MountFuture<T, B>
where
    T: Service<Request<B>>,
    T::Output: IntoOutcome<Request<B>>,
{
    type Output = Outcome<
        <T::Output as IntoOutcome<Request<B>>>::Success,
        <T::Output as IntoOutcome<Request<B>>>::Failure,
        Request<B>,
    >;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let (future, uri) = match this.state.project() {
                StatProj::Init { task, req, path } => {
                    let mut req = req.take().unwrap();
                    if starts_with(path, req.uri()) {
                        let url = req.uri().clone();
                        *req.uri_mut() = replace_path(path, req.uri());
                        ensure_mount(&mut req, path[0..path.len() - 1].to_string());
                        (task.call(req), url)
                    } else {
                        return Poll::Ready(Outcome::Next(req));
                    }
                }
                StatProj::Future { future, uri } => {
                    let ret = match ready!(future.poll(cx)).into_outcome() {
                        Outcome::Next(mut req) => {
                            *req.uri_mut() = uri.take().unwrap();
                            Poll::Ready(Outcome::Next(req))
                        }
                        o => Poll::Ready(o),
                    };

                    return ret;
                }
            };

            self.set(MountFuture {
                state: State::Future {
                    future,
                    uri: Some(uri),
                },
            })
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
#[inline]
fn starts_with(p: &str, url: &Uri) -> bool {
    let path = url.path();
    if path.len() < p.len() {
        path.starts_with(&p[0..(p.len() - 1)])
    } else {
        path.starts_with(p)
    }
}

#[inline]
fn replace_path(path: &str, url: &Uri) -> Uri {
    let p = {
        let path2 = url.path();
        let path = if path2.ends_with("/") {
            &path2[path.len()..]
        } else {
            &path2[(path.len() - 1)..]
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
fn ensure_mount<B>(req: &mut Request<B>, path: String) {
    if req.extensions().get::<MountPath>().is_none() {
        req.extensions_mut().insert(MountPath(Vec::default()));
    }
    req.extensions_mut()
        .get_mut::<MountPath>()
        .unwrap()
        .0
        .push(path);
}
