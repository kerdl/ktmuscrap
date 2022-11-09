use serde_derive::Serialize;
use std::sync::Arc;


use super::Interactor;


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Invoker {
    Auto,
    Manually(Arc<Interactor>)
}

#[derive(Debug)]
pub struct Params {
    pub invoker: Invoker,
}