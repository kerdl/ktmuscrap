use derive_new::new;
use chrono::{NaiveTime, NaiveDate};
use std::ops::Range;

use crate::data::{
    schedule::raw::{
        self,
        table::Cell
    }, 
    weekday::Weekday
};


#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct WeekdayDate {
    pub cell: Cell,
    pub index: usize,
    pub weekday: Weekday,
    pub date: NaiveDate
}

#[derive(new, Debug, Clone)]
pub struct NumTime {
    pub cell: Cell,
    pub index: usize,
    pub num: u32,
    pub time: Range<NaiveTime>,
}

#[derive(new, Debug, Clone)]
pub struct SubjectMapping {
    pub cell: Cell,
    pub group: raw::table::Group,
    pub num_time: NumTime,
    pub weekday_date: WeekdayDate,
}
