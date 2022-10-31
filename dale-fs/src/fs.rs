use dale::{filters::One, Outcome, Service};
use dale_runtime::fs::*;
use futures_core::Future;
use mime::Mime;
use relative_path::RelativePath;
use std::{
    fs::{Metadata, OpenOptions},
    io,
    marker::PhantomData,
    path::PathBuf,
};

use crate::node::FileTypeMask;
use crate::Node;

pub trait FileRequest {
    fn lookup(&self) -> &RelativePath;
}

impl<'a> FileRequest for &'a str {
    fn lookup(&self) -> &RelativePath {
        RelativePath::new(self)
    }
}

#[cfg(feature = "http")]
impl<B> FileRequest for http::Request<B> {
    fn lookup(&self) -> &RelativePath {
        RelativePath::new(self.uri().path())
    }
}

pub struct FileSystem<B: FS> {
    _b: PhantomData<B>,
}

impl<B: FS> FileSystem<B> {
    pub fn root<I>(path: impl Into<PathBuf>) -> impl Service<I> + Clone
    where
        I: FileRequest + Send + Sync,
        B::DirEntry: Send + Sync,
    {
        FileSystem::<B>::root_with(path, FileTypeMask::all())
    }

    pub fn root_with<I>(
        path: impl Into<PathBuf>,
        filetypes: FileTypeMask,
    ) -> impl Service<I, Future = impl Future + Send, Output = Outcome<(I, One<Node>), io::Error, I>>
           + Clone
    where
        I: FileRequest + Send + Sync,
        B::DirEntry: Send + Sync,
    {
        let root = path.into();

        move |req: I| {
            let path = req.lookup().to_path(&root);

            async move {
                if !path.exists() {
                    return Outcome::Next(req);
                }

                let node =
                    dale::fail!(crate::node::read_path::<B>(req.lookup(), &path, filetypes).await);

                Outcome::Success((req, (node,)))
            }
        }
    }

    pub fn file<I>(
        path: impl Into<PathBuf>,
        opts: OpenOptions,
    ) -> impl Service<
        I,
        Future = impl Future + Send,
        Output = Outcome<(I, (B::File, Metadata, Mime)), io::Error, I>,
    > + Clone
    where
        I: Send,
    {
        let path = path.into();
        let mime = mime_guess::from_path(&path).first_or_octet_stream();

        move |req: I| {
            let path = path.clone();
            let opts = opts.clone();
            let mime = mime.clone();
            async move {
                let meta = dale::fail!(B::metadata(&path).await);
                let file = dale::fail!(B::open(&path, opts).await);

                Outcome::Success((req, (file, meta, mime)))
            }
        }
    }
}
