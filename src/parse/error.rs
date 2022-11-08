use thiserror::Error;
use derive_new::new;

use crate::data::schedule::raw;


#[derive(Error, Debug)]
#[error("year is larger than 4 digits")]
pub struct YearTooLarge;

#[derive(new, Error, Debug)]
#[error("no latest html found in for {} (probably there's no htmls)", .sc_type)]
pub struct NoLatest {
    pub sc_type: raw::Type
}

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