use chrono::{NaiveDate, Datelike};

use super::error;
use crate::{REGEX, DynResult};


/// # "SmArT" year parse
/// 
/// Actually it just
/// overwrites numbers at the end
/// of current year with the
/// input
/// 
/// ## Input/output examples
/// - `"2022"` -> `2022`
/// - `"022"` -> 2`022`
/// - `"22"` -> 20`22`
/// - `"2"` -> 202`2`
fn parse_year(year: &str) -> DynResult<i32> {

    if year.len() == 4 {
        return Ok(year.parse::<i32>()?)
    }

    if year.len() > 4 {
        return Err(error::YearTooLarge.into());
    }

    // i.e. assume `year: &str` == "22"

    let now = chrono::Utc::now();

    // "2022"
    let mut now_year_str = format!("{}", now.year());

    //       0..2 (is the len of "22")
    for _ in 0..year.len() {
        // remove last char
        now_year_str.pop();
    }

    // "20"      += "22"
    now_year_str += year;

    // "2022"
    Ok(now_year_str.parse::<i32>().unwrap())
}

/// # Parse `"11.09.02"`, `"11/09/02"`, etc.
pub fn parse_dmy(string: &str) -> Option<NaiveDate> {
    // find something like "11.09.02"
    let matched = REGEX.date.find(string)?.as_str();
    let matched_sep = REGEX.nonword.replace_all(
        matched, "_"
    ).to_string();

    // `d`ay, `m`onth, `y`ear
    let dmy: Vec<&str> = matched_sep.split("_").collect();

    let year = parse_year(dmy.get(2)?).ok()?;
    let month = dmy.get(1)?.parse::<u32>().ok()?;
    let day = dmy.get(0)?.parse::<u32>().ok()?;

    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn remove(string: &str) -> String {
    REGEX.date.replace_all(string, "").trim().to_string()
}