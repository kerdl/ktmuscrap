use std::{error, fmt};

use super::super::schedule;


#[derive(Debug, Clone)]
pub struct ExtractingEmptyContent;
impl fmt::Display for ExtractingEmptyContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "cannot extract empty content")
    }
}
impl error::Error for ExtractingEmptyContent {}

#[derive(Debug, Clone)]
pub struct NotAZipFile {
    pub sc_type: schedule::raw::Type
}
impl NotAZipFile {
    pub fn new(sc_type: schedule::raw::Type) -> NotAZipFile {
        NotAZipFile { sc_type }
    }
}
impl fmt::Display for NotAZipFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not a zip file")
    }
}
impl error::Error for NotAZipFile {}