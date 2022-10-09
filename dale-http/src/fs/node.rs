use dale_fs::{Node, RelativePathBuf};
use http::Uri;
use std::path::Path;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum SerdeNode {
    #[cfg_attr(feature = "serde", serde(rename = "file"))]
    File {
        path: RelativePathBuf,
        #[cfg_attr(feature = "serde", serde(with = "mime_serde_shim"))]
        mime: mime::Mime,
        size: u64,
        #[cfg_attr(feature = "serde", serde(with = "http_serde::uri"))]
        href: Uri,
    },
    #[cfg_attr(feature = "serde", serde(rename = "directory"))]
    Dir {
        path: RelativePathBuf,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
        children: Vec<SerdeNode>,
        #[cfg_attr(feature = "serde", serde(with = "http_serde::uri"))]
        href: Uri,
    },
}

impl SerdeNode {
    fn create_uri(host: &str, path: &str, mount: &Option<String>) -> Uri {
        let mut uri = Uri::builder().scheme("http").authority(host);

        if let Some(mount) = mount {
            let path = Path::new(mount).join(path);
            uri = uri.path_and_query(path.to_string_lossy().as_ref());
        } else {
            uri = uri.path_and_query(path)
        }

        uri.build().unwrap()
    }
    pub(crate) fn from(node: Node, host: &str, mount: &Option<String>) -> SerdeNode {
        match node {
            Node::Dir(dir) => {
                // let path = dir.path.into_os_string().to_string_lossy().to_string();
                let uri = SerdeNode::create_uri(host, dir.path.as_str(), mount);
                SerdeNode::Dir {
                    path: dir.path,
                    href: uri,
                    children: dir
                        .children
                        .into_iter()
                        .map(|m| SerdeNode::from(m, host, mount))
                        .collect(),
                }
            }
            Node::File(file) => {
                // let path = file.path.into_os_string().to_string_lossy().to_string();
                let uri = SerdeNode::create_uri(host, file.path.as_str(), mount);
                SerdeNode::File {
                    path: file.path,
                    href: uri,
                    mime: file.mime,
                    size: file.meta.len(),
                }
            }
        }
    }
}

// pub fn serve_json<T: Backend, B: Body, P: Into<PathBuf>>(
//     path: P,
// ) -> impl Service<
//     Request<B>,
//     Output = impl Reply<B>,
//     Error = Reject<Request<B>, Error>,
//     Future = impl Future + Send,
// > + Clone
// where
//     T: Send,
//     <<T as Backend>::FS as FS>::DirEntry: Send + Sync,
//     B: Send + 'static,
// {
//     let service =
//         mdv_fs::FileSystem::<T>::root_with(path, FileTypeMask::Directory | FileTypeMask::Regular);

//     service
//         .and(filters::ext())
//         .and(filters::header::optional())
//         .map(
//             |path: Node, mount_path: Option<crate::MountPath>, host: Option<Host>| {
//                 let host = host
//                     .map(|m| m.to_string())
//                     .unwrap_or_else(|| "localhost".to_string());
//                 SerdeNode::from(path, &host, &mount_path.map(|m| m.to_string()))
//             },
//         )
//         .map(reply::json)
// .map_err(|err| {
//     //
//     match err {
//         Either::A(a) => {
//             //
//             match a {
//                 Either::A(i) => match i {
//                     Reject::Err(err) => Reject::Err(Error::new(err)),
//                     Reject::Reject(req, err) => Reject::Reject(req, Error::new(err)),
//                 },
//                 Either::B(b) => Reject::Err(b),
//             }
//         }
//         Either::B(b) => {
//             //
//             Reject::Err(b)
//         }
//     }
// })
// }
