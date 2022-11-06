use std::ops::Range;

use chrono::NaiveTime;
use serde::Serialize;

use crate::data::schedule as regular;
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    pub compared: Primitive<regular::Subject>,

    pub teachers: Changes<String>,
    pub cabinet: Primitive<Option<String>>,
    pub time: Primitive<Range<NaiveTime>>
}
impl DetailedCmp<regular::Subject, Subject> for Subject {
    fn compare(
        old: regular::Subject,
        new: regular::Subject
    ) -> Subject {

        let compared = Primitive::new(old.clone(), new.clone());

        let teachers = Changes::compare(
            old.teachers, 
            new.teachers
        );
        let cabinet = Primitive::new(
            old.cabinet,
            new.cabinet
        );
        let time = Primitive::new(
            old.time.clone(),
            new.time.clone()
        );

        Subject {
            compared,
            teachers,
            cabinet,
            time
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Day {
    pub compared: Primitive<regular::Day>,

    pub subjects: DetailedChanges<regular::Subject, Subject>
}
impl DetailedCmp<regular::Day, Day> for Day {
    fn compare(
        old: regular::Day,
        new: regular::Day
    ) -> Day {

        let compared = Primitive::new(old.clone(), new.clone());
        let subjects = DetailedChanges::compare(
            old.subjects.clone(),
            new.subjects.clone()
        );

        Day { compared, subjects }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub compared: Primitive<regular::Group>,

    pub days: DetailedChanges<regular::Day, Day>
}
impl DetailedCmp<regular::Group, Group> for Group {
    fn compare(
        old: regular::Group,
        new: regular::Group
    ) -> Group {

        let compared = Primitive::new(old.clone(), new.clone());
        let days = DetailedChanges::compare(
            old.days.clone(),
            new.days.clone()
        );

        Group { compared, days }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub compared: Primitive<regular::Page>,

    pub groups: DetailedChanges<regular::Group, Group>
}
impl DetailedCmp<regular::Page, Page> for Page {
    fn compare(
        old: regular::Page,
        new: regular::Page
    ) -> Page {

        let compared = Primitive::new(old.clone(), new.clone());
        let groups = DetailedChanges::compare(
            old.groups.clone(),
            new.groups.clone()
        );

        Page { compared, groups }
    }
}