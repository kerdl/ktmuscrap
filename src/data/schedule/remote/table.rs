use chrono::{NaiveTime, NaiveDate};
use std::ops::Range;

use crate::data::{
    schedule::raw::table::Cell, 
    weekday::Weekday
};


#[derive(Debug, Clone)]
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
pub struct SubjectMapping {
    pub cell: Cell,
    pub group: String,
    pub num_time: NumTime,
    pub weekday_date: WeekdayDate,
    pub subject: String,
}
impl SubjectMapping {
    pub fn new(
        cell: Cell,
        group: String,
        num_time: NumTime, 
        weekday_date: WeekdayDate, 
        subject: String, 
    ) -> SubjectMapping {

        SubjectMapping { cell, group, num_time, weekday_date, subject }
    }
}