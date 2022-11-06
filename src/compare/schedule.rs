use std::ops::Range;

use chrono::NaiveTime;

use crate::data;
use super::{Changes, Compare};


pub struct Subject {
    teachers: Changes<String>,
    cabinet: Compare<String>,
    time: Compare<Range<NaiveTime>>
}
impl Subject {
    pub fn compare(
        old: data::schedule::Subject,
        new: data::schedule::Subject
    ) -> Subject {

        todo!();
        /*
        Subject {
            teachers,
            cabinet,
            time
        }
        */
    }
}

pub struct Day {
    subjects: Changes<Subject>
}

pub struct Group {
    days: Changes<Day>
}

pub struct Page {
    groups: Changes<Group>
}