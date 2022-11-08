use thiserror::Error;
use crate::data::schedule::raw;


#[derive(Error, Debug)]
#[error("r_weekly's date range don't have ft_daily's date")]
pub struct FtDateIsNotInRWeeklyRange {
    pub latest: raw::Type,
}

#[derive(Error, Debug)]
#[error("ft_weekly and r_weekly have different weeks and cannot be merged")]
pub struct DifferentWeeks {
    pub latest: raw::Type,
}
