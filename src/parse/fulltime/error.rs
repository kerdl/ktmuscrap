use derive_new::new;
use thiserror::Error;

use crate::data::schedule::raw;


#[derive(new, Error, Debug)]
#[error("{} does not contains tables", .sc_type)]
pub struct NoTables {
    pub sc_type: raw::Type
}

#[derive(new, Error, Debug)]
#[error("{} tables cannot be mapped", .sc_type)]
pub struct NoMappings {
    pub sc_type: raw::Type
}