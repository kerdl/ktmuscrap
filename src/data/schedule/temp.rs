//! # Temp storages

use chrono::NaiveTime;
use std::ops::Range;


#[derive(Debug, Clone)]
pub struct NumTimeIndex {
    pub num: u32,
    pub time: Range<NaiveTime>,
    pub index: usize,
}
impl NumTimeIndex {
    pub fn new(num: u32, time: Range<NaiveTime>, index: usize, ) -> NumTimeIndex {
        NumTimeIndex { num, time, index }
    }
}