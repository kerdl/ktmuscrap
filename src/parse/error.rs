use thiserror::Error;


#[derive(Error, Debug)]
#[error("year is larger than 4 digits")]
pub struct YearTooLarge;
