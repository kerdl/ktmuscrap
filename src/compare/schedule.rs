use async_trait::async_trait;
use chrono::NaiveTime;
use serde::Serialize;
use std::ops::Range;

use crate::data::{schedule as regular, Weekday};
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num: Option<Primitive<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teachers: Option<Changes<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabinet: Option<Primitive<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Primitive<Range<NaiveTime>>>
}
#[async_trait]
impl DetailedCmp<regular::Subject, Subject> for Subject {
    async fn compare(
        old: Option<regular::Subject>,
        new: regular::Subject
    ) -> Subject {

        let name = new.name.clone();
        let num = Primitive::new(
            old.as_ref().map(|old| old.num),
            new.num
        );
        let teachers = Changes::compare(
            old.as_ref().map(|old| old.teachers.clone()),
            new.teachers
        ).await;
        let cabinet = Primitive::new(
            old.as_ref().map(|old| old.cabinet.clone()),
            new.cabinet
        );
        let time = Primitive::new(
            old.as_ref().map(|old| old.time.clone()),
            new.time.clone()
        );

        Subject {
            name,
            num: if num.is_different_hash() {
                Some(num)
            } else {
                None
            },
            teachers: if teachers.any_changes() {
                Some(teachers)
            } else {
                None
            },
            cabinet: if cabinet.is_different_hash() {
                Some(cabinet)
            } else {
                None
            },
            time: if time.is_different_hash() {
                Some(time)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Day {
    pub weekday: Weekday,
    pub subjects: DetailedChanges<regular::Subject, Subject>
}
#[async_trait]
impl DetailedCmp<regular::Day, Day> for Day {
    async fn compare(
        old: Option<regular::Day>,
        new: regular::Day
    ) -> Day {

        let weekday = new.weekday.clone();
        let subjects = DetailedChanges::compare(
            old.map(|old| old.subjects.clone()),
            new.subjects.clone()
        ).await;

        Day { weekday, subjects }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub name: String,
    pub days: DetailedChanges<regular::Day, Day>
}
#[async_trait]
impl DetailedCmp<regular::Group, Group> for Group {
    async fn compare(
        old: Option<regular::Group>,
        new: regular::Group
    ) -> Group {

        let name = new.name.clone();
        let days = DetailedChanges::compare(
            old.map(|old| old.days.clone()),
            new.days.clone()
        ).await;

        Group { name, days }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub raw: String,
    pub groups: DetailedChanges<regular::Group, Group>
}
#[async_trait]
impl DetailedCmp<regular::Page, Page> for Page {
    async fn compare(
        old: Option<regular::Page>,
        new: regular::Page
    ) -> Page {

        let raw = new.raw.clone();
        let groups = DetailedChanges::compare(
            old.map(|old| old.groups.clone()),
            new.groups.clone()
        ).await;

        Page { raw, groups }
    }
}