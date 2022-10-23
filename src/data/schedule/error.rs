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
