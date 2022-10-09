use std::{
    fs::{Metadata, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use async_compat::Compat;
use async_trait::async_trait;
use tokio_stream::wrappers::ReadDirStream;

use crate::fs::{DirEntry, File, FS};

#[async_trait]
impl File for Compat<tokio::fs::File> {
    async fn metadata(&self) -> Result<Metadata, io::Error> {
        self.metadata().await
    }
}

#[async_trait]
impl FS for super::Tokio {
    type DirEntry = tokio::fs::DirEntry;
    type ReadDir = ReadDirStream;

    type File = Compat<tokio::fs::File>;

    async fn open<P: AsRef<Path> + Send>(path: P, opts: OpenOptions) -> io::Result<Self::File> {
        Ok(Compat::new(
            tokio::fs::OpenOptions::from(opts).open(path).await?,
        ))
    }

    async fn read_dir<P: AsRef<Path> + Send>(path: P) -> io::Result<Self::ReadDir> {
        Ok(ReadDirStream::new(tokio::fs::read_dir(path).await?))
    }
    async fn read<P: AsRef<Path> + Send>(path: P) -> io::Result<Vec<u8>> {
        tokio::fs::read(path).await
    }

    async fn metadata<P: AsRef<Path> + Send>(path: P) -> io::Result<Metadata> {
        tokio::fs::metadata(path).await
    }
}

#[async_trait]
impl DirEntry for tokio::fs::DirEntry {
    fn path(&self) -> PathBuf {
        self.path()
    }
    async fn metadata(&self) -> io::Result<Metadata> {
        self.metadata().await
    }
}
