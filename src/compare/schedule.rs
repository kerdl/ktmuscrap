use async_trait::async_trait;
use chrono::{NaiveTime, NaiveDate};
use serde::Serialize;
use std::ops::{Range, RangeInclusive};

use crate::data::{schedule as regular, Weekday};
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    pub name: Option<String>,
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
        new: Option<regular::Subject>
    ) -> Subject {
        let name = new.as_ref().map(|new| new.name.clone());
        let num = Primitive::new(
            old.as_ref().map(|old| old.num),
            new.as_ref().map(|new| new.num),
        );
        let teachers = Changes::compare(
            old.as_ref().map(|old| old.teachers.clone()),
            new.as_ref().map(|new| new.teachers.clone()),
        ).await;
        let cabinet = Primitive::new(
            old.as_ref().map(|old| old.cabinet.clone()),
            new.as_ref().map(|new| new.cabinet.clone()),
        );
        let time = Primitive::new(
            old.as_ref().map(|old| old.time.clone()),
            new.as_ref().map(|new| new.time.clone()),
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
    pub weekday: Option<Weekday>,
    pub subjects: DetailedChanges<regular::Subject, Subject>
}
#[async_trait]
impl DetailedCmp<regular::Day, Day> for Day {
    async fn compare(
        old: Option<regular::Day>,
        new: Option<regular::Day>
    ) -> Day {
        let weekday = new.as_ref().map(|new| new.weekday.clone());
        let subjects = DetailedChanges::compare(
            old.map(|old| old.subjects.clone()),
            new.map(|new| new.subjects.clone()),
        ).await;

        Day { weekday, subjects }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub name: Option<String>,
    pub days: DetailedChanges<regular::Day, Day>
}
#[async_trait]
impl DetailedCmp<regular::Group, Group> for Group {
    async fn compare(
        old: Option<regular::Group>,
        new: Option<regular::Group>
    ) -> Group {
        if new.as_ref().unwrap().name == "2КДД40" {
            println!("");
        }

        let name = new.as_ref().map(|new| new.name.clone());
        let days = DetailedChanges::compare(
            old.map(|old| old.days.clone()),
            new.map(|new| new.days.clone()),
        ).await;

        Group { name, days }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub raw: Option<String>,
    pub date: Primitive<RangeInclusive<NaiveDate>>,
    pub groups: DetailedChanges<regular::Group, Group>
}
#[async_trait]
impl DetailedCmp<regular::Page, Page> for Page {
    async fn compare(
        old: Option<regular::Page>,
        new: Option<regular::Page>
    ) -> Page {
        let raw = new.as_ref().map(|new| new.raw.clone());
        let date = Primitive::new(
            old.as_ref().map(|old| old.date.clone()),
            new.as_ref().map(|new| new.date.clone()),
        );
        let groups = DetailedChanges::compare(
            old.as_ref().map(|old| old.groups.clone()),
            new.as_ref().map(|new| new.groups.clone()),
        ).await;

        Page { raw, date, groups }
    }
}
