use bytes::Bytes;
use std::path::PathBuf;


#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub bytes: Bytes
}
