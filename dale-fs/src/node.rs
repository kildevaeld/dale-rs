use bitflags::bitflags;
use dale_runtime::fs::{DirEntry, FS};
use futures_util::StreamExt;
use pin_utils::pin_mut;
use relative_path::{RelativePath, RelativePathBuf};
use std::{fs::Metadata, io, path::Path, str::FromStr};

bitflags! {
    #[repr(transparent)]
    pub struct FileTypeMask: u8 {
        const REGULAR = 1<<0;
        const SYMLINK = 1<<1;
        const DIRECTORY = 1<<2;
    }
}

impl From<std::fs::FileType> for FileTypeMask {
    fn from(t: std::fs::FileType) -> Self {
        if t.is_symlink() {
            FileTypeMask::SYMLINK
        } else if t.is_file() {
            FileTypeMask::REGULAR
        } else {
            FileTypeMask::DIRECTORY
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub mime: mime::Mime,
    pub meta: Metadata,
    pub path: RelativePathBuf,
}

#[derive(Debug, Clone)]
pub struct Dir {
    pub children: Vec<Node>,
    pub path: RelativePathBuf,
}

#[derive(Debug, Clone)]
pub enum Node {
    File(File),
    Dir(Dir),
}

macro_rules! read_file {
    ($path: expr,$meta: expr) => {{
        let mime = match $path.extension() {
            Some(ext) => mime_guess::from_ext(ext).first_or_octet_stream(),
            None => mime::Mime::from_str("application/octet-stream").unwrap(),
        };

        Result::<_, io::Error>::Ok(File {
            mime,
            meta: $meta,
            path: $path,
        })
    }};
}

pub async fn read_path<F: FS>(
    root: &RelativePath,
    path: &Path,
    filetypes: FileTypeMask,
) -> Result<Node, io::Error> {
    let metadata = F::metadata(path).await?;

    let filetype = metadata.file_type().into();
    if !filetypes.contains(filetype) {
        return Err(io::ErrorKind::NotFound.into());
    }

    let node = if metadata.is_dir() {
        let stream = F::read_dir(path).await?;
        pin_mut!(stream);

        let mut out = Vec::default();
        while let Some(path) = stream.next().await {
            let path = match path {
                Ok(path) => path,
                Err(_) => {
                    continue;
                }
            };
            let meta = F::metadata(path.path()).await?;

            let filetype = meta.file_type().into();
            if !filetypes.contains(filetype) {
                continue;
            }

            let path = RelativePathBuf::from_path(path.path()).unwrap();

            let node = if meta.is_dir() {
                Node::Dir(Dir {
                    children: Vec::default(),
                    path,
                })
            } else {
                Node::File(read_file!(path, meta)?)
            };

            out.push(node);
        }
        Node::Dir(Dir {
            children: out,
            path: root.to_relative_path_buf(),
        })
    } else {
        Node::File(read_file!(root.to_relative_path_buf(), metadata)?)
    };

    Ok(node)
}
