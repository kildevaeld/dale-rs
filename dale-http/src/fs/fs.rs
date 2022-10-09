use super::file_conditional::file_conditional;
use super::file_options::file_options;
use crate::error::Error;
use crate::filters;
use crate::{modifier::Modifier, Body};
use dale::filters::One;
use dale::{Outcome, Service, ServiceExt};
use dale_fs::{FileTypeMask, Node};
use dale_runtime::fs::FS;
use dale_runtime::Tokio;
use futures_core::Future;
use headers::Host;
use http::{Request, Response};

pub use super::node::*;
use std::{fs::OpenOptions, path::PathBuf};

pub fn root<B>(
    path: impl Into<PathBuf>,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, One<SerdeNode>), Error, Request<B>>,
> + Clone
where
    B: Send + Sync + 'static,
{
    root_with(path, FileTypeMask::REGULAR | FileTypeMask::DIRECTORY)
}

pub fn root_with<B>(
    path: impl Into<PathBuf>,
    filetypes: FileTypeMask,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, One<SerdeNode>), Error, Request<B>>,
> + Clone
where
    B: Send + Sync + 'static,
{
    dale_fs::FileSystem::<Tokio>::root_with(path, filetypes)
        .and(filters::ext())
        .and(filters::header::optional())
        .map(
            |path: Node, mount_path: Option<crate::mount::MountPath>, host: Option<Host>| {
                let host = host
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "localhost".to_string());
                SerdeNode::from(path, &host, &mount_path.map(|m| m.to_string()))
            },
        )
        .err_into()
}

pub fn file<B>(
    path: impl Into<PathBuf>,
) -> impl Service<Request<B>, Future = impl Future + Send, Output = crate::Outcome<B>> + Clone
where
    B: Body + Modifier<Response<B>> + Send + Sync + 'static,
{
    let mut opts = OpenOptions::new();
    opts.read(true);

    dale_fs::FileSystem::<Tokio>::file(path, opts)
        .and(file_options())
        .then(|(_, (node, meta, mime, options))| async move {
            Result::<_, Error>::Ok(file_conditional(node, mime, meta, options)?)
        })
        .err_into()
}

pub fn dir<B>(
    path: impl Into<PathBuf>,
) -> impl Service<Request<B>, Future = impl Future + Send, Output = crate::Outcome<B>> + Clone
where
    B: Body + Modifier<Response<B>> + Send + Sync + 'static,
{
    let mut opts = OpenOptions::new();
    opts.read(true);

    let path = path.into();

    let service = dale_fs::FileSystem::<Tokio>::root_with(path.clone(), FileTypeMask::REGULAR)
        .and(file_options())
        .then(move |(_, (node, options))| {
            let root = path.clone();
            async move {
                match node {
                    Node::File(meta) => {
                        //
                        let path = meta.path.to_path(root);
                        let mut o = OpenOptions::new();
                        o.read(true);
                        let file = Tokio::open(path, o).await?;

                        let ret = file_conditional(file, meta.mime, meta.meta, options);

                        ret
                    }
                    _ => {
                        todo!();
                    }
                }
            }
        })
        .err_into();

    service
}