use thiserror::Error;
use crate::data::schedule::Page;


#[derive(Error, Debug)]
#[error("merge error")]
pub enum MergeError<'a> {
    InvalidKind(&'a Page),
    NonOverlappingDates(NonOverlappingDates<'a>)
}

#[derive(Error, Debug)]
#[error("pages have non-overlapping date ranges")]
pub struct NonOverlappingDates<'a> {
    pub latest: &'a Page
}
