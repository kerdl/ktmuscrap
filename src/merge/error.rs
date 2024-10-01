use thiserror::Error;
use crate::data::schedule::Page;


#[derive(Error, Debug)]
#[error("merge error")]
pub enum MergeError<'a> {
    InvalidKind(&'a Page),
    NonOverlappingDates
}
