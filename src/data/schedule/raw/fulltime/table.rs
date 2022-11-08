use derive_new::new;
use chrono::{NaiveTime, NaiveDate};
use std::ops::Range;

use crate::data::{
    weekday::Weekday,
    schedule::raw
};


pub enum CellType {
    Num,
    Time,
    Subject
}
impl CellType {
    pub fn from_index(index: usize) -> CellType {
        match index {
            0 => CellType::Num,
            1 => CellType::Time,
            _ => CellType::Subject
        }
    }
}

#[derive(new, Debug, Clone)]
pub struct NumTime {
    pub num: u32,
    pub time: Range<NaiveTime>,
}

#[derive(new, Debug, Clone)]
pub struct WeekdayWithOrigin {
    pub raw: String,
    pub guessed: Weekday,
}

#[derive(new, Debug, Clone)]
pub struct SubjectMapping {
    pub name: String,
    pub weekday: WeekdayWithOrigin,
    pub num_time: NumTime,
}
impl SubjectMapping {
    pub fn is_empty(&self) -> bool {
        let is_no_chars = self.name.is_empty();
        let only_dashes = self.name.chars().all(|char| { char == '-'});

        is_no_chars || only_dashes
    }
}

#[derive(new, Debug, Clone)]
pub struct GroupSubjects {
    pub group: raw::table::Group,
    pub date_range: Range<NaiveDate>,
    pub subjects: Vec<SubjectMapping>,
}