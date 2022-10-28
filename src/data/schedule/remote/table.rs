use chrono::NaiveTime;
use std::ops::Range;

use crate::data::schedule::raw::table::Cell;


#[derive(Debug, Clone)]
pub struct NumTime {
    pub cell: Cell,
    pub num: u32,
    pub time: Range<NaiveTime>,
}
impl NumTime {
    pub fn new(
        cell: Cell,
        num: u32,
        time: Range<NaiveTime>,
    ) -> NumTime {

        NumTime {
            cell,
            num,
            time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    pub cell: Cell,
    pub valid: String
}
impl Group {
    pub fn new(cell: Cell, valid: String, ) -> Group {
        Group { cell, valid }
    }

    pub fn schedule_start_index(&self) -> usize {
        self.cell.x_index + 1
    }
}