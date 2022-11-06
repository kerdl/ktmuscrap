use std::ops::Range;

use chrono::NaiveTime;

use crate::data::schedule as regular;
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


#[derive(Debug)]
pub struct Subject<'a> {
    pub old: &'a regular::Subject,
    pub new: &'a regular::Subject,

    pub teachers: Changes<&'a String>,
    pub cabinet: Primitive<&'a Option<String>>,
    pub time: Primitive<&'a Range<NaiveTime>>
}
impl<'a> DetailedCmp<&'a regular::Subject, Subject<'a>> for Subject<'a> {
    fn compare(
        old: &'a regular::Subject,
        new: &'a regular::Subject
    ) -> Subject<'a> {

        let teachers = Changes::compare(
            &old.teachers, 
            &new.teachers
        );
        let cabinet = Primitive::new(
            &old.cabinet,
            &new.cabinet
        );
        let time = Primitive::new(
            &old.time,
            &new.time
        );

        Subject {
            old,
            new,
            teachers,
            cabinet,
            time
        }
    }
}

#[derive(Debug)]
pub struct Day<'a> {
    pub old: &'a regular::Day,
    pub new: &'a regular::Day,

    pub subjects: DetailedChanges<&'a regular::Subject, Subject<'a>>
}
impl<'a> DetailedCmp<&'a regular::Day, Day<'a>> for Day<'a> {
    fn compare(
        old: &'a regular::Day,
        new: &'a regular::Day
    ) -> Day<'a> {

        let subjects = DetailedChanges::compare(
            &old.subjects,
            &new.subjects
        );

        Day { old, new, subjects }
    }
}

#[derive(Debug)]
pub struct Group<'a> {
    pub old: &'a regular::Group,
    pub new: &'a regular::Group,

    pub days: DetailedChanges<&'a regular::Day, Day<'a>>
}
impl<'a> DetailedCmp<&'a regular::Group, Group<'a>> for Group<'a> {
    fn compare(
        old: &'a regular::Group,
        new: &'a regular::Group
    ) -> Group<'a> {

        let days = DetailedChanges::compare(
            &old.days,
            &new.days
        );

        Group { old, new, days }
    }
}

#[derive(Debug)]
pub struct Page<'a> {
    pub old: &'a regular::Page,
    pub new: &'a regular::Page,

    pub groups: DetailedChanges<&'a regular::Group, Group<'a>>
}
impl<'a> DetailedCmp<&'a regular::Page, Page<'a>> for Page<'a> {
    fn compare(
        old: &'a regular::Page,
        new: &'a regular::Page
    ) -> Page<'a> {

        let groups = DetailedChanges::compare(
            &old.groups,
            &new.groups
        );

        Page { old, new, groups }
    }
}