use chrono::{NaiveTime, NaiveDate};
use std::ops::Range;

use crate::data::{
    schedule::raw::table::Cell, 
    weekday::Weekday
};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekdayDate {
    pub cell: Cell,
    pub index: usize,
    pub weekday: Weekday,
    pub date: NaiveDate
}
impl WeekdayDate {
    pub fn new(
        cell: Cell, 
        index: usize,
        weekday: Weekday, 
        date: NaiveDate
    ) -> WeekdayDate {

        WeekdayDate { cell, index, weekday, date, }
    }
}

#[derive(Debug, Clone)]
pub struct NumTime {
    pub cell: Cell,
    pub index: usize,
    pub num: u32,
    pub time: Range<NaiveTime>,
}
impl NumTime {
    pub fn new(
        cell: Cell,
        index: usize,
        num: u32,
        time: Range<NaiveTime>,
    ) -> NumTime {

        NumTime { cell, index, num, time, }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub raw: String,
    pub valid: String,
}
impl Group {
    pub fn new(raw: String, valid: String, ) -> Group {
        Group { raw, valid }
    }
}

#[derive(Debug, Clone)]
pub struct SubjectMapping {
    pub cell: Cell,
    pub group: Group,
    pub num_time: NumTime,
    pub weekday_date: WeekdayDate,
}
impl SubjectMapping {
    pub fn new(
        cell: Cell,
        group: Group,
        num_time: NumTime, 
        weekday_date: WeekdayDate, 
    ) -> SubjectMapping {

        SubjectMapping { cell, group, num_time, weekday_date }
    }
}