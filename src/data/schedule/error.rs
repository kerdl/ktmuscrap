use thiserror::Error;


#[derive(Error, Debug)]
#[error("cannot extract empty content")]
pub struct ExtractingEmptyContent;
