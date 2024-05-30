use serde_derive::Serialize;
use bytes::Bytes;
use std::{path::PathBuf, sync::Arc, collections::HashMap};


use super::Interactor;


#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub bytes: Bytes
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Invoker {
    Auto,
    Manually(Arc<Interactor>)
}

#[derive(Debug)]
pub struct Params {
    pub invoker: Invoker
}