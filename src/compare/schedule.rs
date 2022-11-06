use std::ops::Range;

use chrono::NaiveTime;

use crate::data::schedule as regular;
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


pub struct Subject<'a> {
    teachers: Changes<&'a String>,
    cabinet: Primitive<&'a Option<String>>,
    time: Primitive<&'a Range<NaiveTime>>
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
            teachers,
            cabinet,
            time
        }
    }

    /* 
    pub fn compare_vec(
        vector: &'a Vec<Primitive<&regular::Subject>>
    ) -> Vec<Subject<'a>> {

        let mut compared = vec![];

        for changed in vector.iter() {
            let subject = Subject::compare(
                changed.old,
                changed.new
            );
            compared.push(subject);
        }

        compared
    }
    */
}

pub struct Day<'a> {
    subjects: DetailedChanges<&'a regular::Subject, Subject<'a>>
}
impl<'a> Day<'a> {
    pub fn compare(
        old: &'a regular::Day,
        new: &'a regular::Day
    ) -> Day<'a> {

        todo!()
        /* 
        let subjects = DetailedChanges::compare(
            &old.subjects,
            &new.subjects
        );

        Day { subjects }
        */
    }
}

pub struct Group<'a> {
    days: DetailedChanges<&'a regular::Day, Day<'a>>
}

pub struct Page<'a> {
    groups: DetailedChanges<&'a regular::Group, Group<'a>>
}