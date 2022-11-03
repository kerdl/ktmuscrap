use thiserror::Error;

#[derive(Error, Debug)]
#[error("r_weekly's date range don't have ft_daily's date")]
pub struct FtDateIsNotInRWeeklyRange;

#[derive(Error, Debug)]
#[error("ft_weekly and r_weekly have different weeks and cannot be merged")]
pub struct DifferentWeeks;
