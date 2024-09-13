use chrono::{Datelike, NaiveDate};
use crate::regexes;


#[derive(thiserror::Error, Debug)]
#[error("year parsing error")]
pub enum YearError {
    InvalidLength(usize),
    InvalidValue(std::num::ParseIntError)
}
impl From<std::num::ParseIntError> for YearError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidValue(value)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("month parsing error")]
pub enum MonthError {
    InvalidLength(usize),
    InvalidValue(std::num::ParseIntError),
    TooLarge
}
impl From<std::num::ParseIntError> for MonthError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidValue(value)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("day parsing error")]
pub enum DayError {
    InvalidLength(usize),
    InvalidValue(std::num::ParseIntError),
    TooLarge
}
impl From<std::num::ParseIntError> for DayError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::InvalidValue(value)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("date parsing error")]
pub enum WholeError {
    NotEnoughData,
    TooMuchData,
    DoesNotExist,
    DayError(DayError),
    MonthError(MonthError),
    YearError(YearError)
}
impl From<Option<NaiveDate>> for WholeError {
    fn from(value: Option<NaiveDate>) -> Self {
        Self::DoesNotExist
    }
}
impl From<DayError> for WholeError {
    fn from(value: DayError) -> Self {
        Self::DayError(value)
    }
}
impl From<MonthError> for WholeError {
    fn from(value: MonthError) -> Self {
        Self::MonthError(value)
    }
}
impl From<YearError> for WholeError {
    fn from(value: YearError) -> Self {
        Self::YearError(value)
    }
}


/// # Smart year parse
/// 
/// ## Input/output examples
/// - `"2024"` -> `2024`
/// - `"024"` -> 2`024`
/// - `"24"` -> 20`24`
/// - `"4"` -> 202`4`
pub fn year(string: &str) -> Result<i32, YearError> {
    // e.g. assume year == "24"
    if string.len() == 4 {
        return Ok(string.parse::<i32>()?);
    }
    if string.len() > 4 {
        return Err(YearError::InvalidLength(string.len()));
    }
    let now = chrono::Utc::now();
    // "2024"
    let mut now_year_str = format!("{}", now.year());
    //       0..2 (is the length of "24")
    for _ in 0..string.len() {
        // remove last char
        now_year_str.pop();
    }
    // "20"      += "24"
    now_year_str += string;
    // "2024"
    Ok(now_year_str.parse::<i32>().unwrap())
}

pub fn month(string: &str) -> Result<u32, MonthError> {
    if string.len() != 2 {
        return Err(MonthError::InvalidLength(string.len()));
    }
    let num = string.parse::<u32>()?;
    if num > 12 {
        return Err(MonthError::TooLarge)
    }
    Ok(num)
}

pub fn day(string: &str) -> Result<u32, DayError> {
    if string.len() != 2 {
        return Err(DayError::InvalidLength(string.len()));
    }
    let num = string.parse::<u32>()?;
    if num > 32 {
        return Err(DayError::TooLarge)
    }
    Ok(num)
}

pub fn whole(date: &str) -> Result<NaiveDate, WholeError> {
    let parts = regexes().nonword.split(date).collect::<Vec<&str>>();
    if parts.len() < 2 {
        return Err(WholeError::NotEnoughData)
    } else if parts.len() > 3 {
        return Err(WholeError::TooMuchData)
    }

    if parts.len() == 2 {
        let d = parts.get(0).unwrap();
        let m = parts.get(1).unwrap();
        let y = chrono::Utc::now().year();

        return Ok(
            NaiveDate::from_ymd_opt(y, month(m)?, day(d)?)
            .ok_or(WholeError::DoesNotExist)?
        )
    } else if parts.len() == 3 {
        let d = parts.get(0).unwrap();
        let m = parts.get(1).unwrap();
        let y = parts.get(2).unwrap();

        return Ok(
            NaiveDate::from_ymd_opt(year(y)?, month(m)?, day(d)?)
            .ok_or(WholeError::DoesNotExist)?
        )
    }

    unreachable!();
}