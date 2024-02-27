use chrono::NaiveTime;
use std::ops::Range;

use crate::REGEX;


pub fn parse_range_hm(string: &str) -> Option<Range<NaiveTime>> {
    let time_match = REGEX.time.find(string)?.as_str();

    let start_end = REGEX.time_sep.split(&time_match).collect::<Vec<&str>>();
    // let start_end = time_match.split("-").collect::<Vec<&str>>();
    let start = start_end.get(0).unwrap();
    let end = start_end.get(1).unwrap();

    // `h`ours `m`inutes
    let start_hm: Vec<&str> = start.split(":").collect();
    let end_hm: Vec<&str> = end.split(":").collect();

    let start_hour = start_hm.get(0).unwrap().parse::<u32>().ok()?;
    let start_minute = start_hm.get(1).unwrap().parse::<u32>().ok()?;
    let end_hour = end_hm.get(0).unwrap().parse::<u32>().ok()?;
    let end_minute = end_hm.get(1).unwrap().parse::<u32>().ok()?;

    let start = NaiveTime::from_hms(start_hour, start_minute, 0);
    let end = NaiveTime::from_hms(end_hour, end_minute, 0);

    Some(start..end)
}

pub fn remove(string: &str) -> String {
    REGEX.time.replace_all(string, "").trim().to_string()
}