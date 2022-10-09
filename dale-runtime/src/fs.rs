use async_trait::async_trait;
use futures_core::Stream;
use futures_io::{AsyncRead, AsyncSeek, AsyncWrite};
use std::{
    fs::{Metadata, OpenOptions},
    io,
    path::{Path, PathBuf},
};

#[async_trait]
pub trait File: AsyncRead + AsyncWrite + AsyncSeek {
    async fn metadata(&self) -> Result<Metadata, std::io::Error>;
}

#[async_trait]
pub trait DirEntry {
    fn path(&self) -> PathBuf;
    async fn metadata(&self) -> io::Result<Metadata>;
}

#[async_trait]
pub trait FS {
    type ReadDir: Stream<Item = Result<Self::DirEntry, io::Error>> + Send;
    type DirEntry: DirEntry;
    type File: File;

    async fn open<P: AsRef<Path> + Send>(path: P, opts: OpenOptions) -> io::Result<Self::File>;

    async fn read_dir<P: AsRef<Path> + Send>(path: P) -> io::Result<Self::ReadDir>;
    async fn read<P: AsRef<Path> + Send>(path: P) -> io::Result<Vec<u8>>;
    async fn metadata<P: AsRef<Path> + Send>(path: P) -> io::Result<Metadata>;
}
