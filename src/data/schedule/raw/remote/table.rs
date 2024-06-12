use derive_new::new;
use chrono::{NaiveTime, NaiveDate};
use std::ops::Range;

use crate::data::{
    schedule::raw::{
        self,
        NumTime,
        table::{Cell, Teacher as TeacherDescriptor}
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

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct Teacher {
    pub cell: Cell,
    pub index: usize,
    pub teacher: TeacherDescriptor
}

#[derive(new, Debug, Clone)]
pub struct SubjectMapping {
    pub cell: Cell,
    pub group: raw::table::Group,
    pub num_time: NumTime,
    pub weekday_date: WeekdayDate,
}

#[derive(new, Debug, Clone)]
pub struct TchrSubjectMapping {
    pub cell: Cell,
    pub teacher: raw::table::Teacher,
    pub num_time: NumTime,
    pub weekday_date: WeekdayDate,
}