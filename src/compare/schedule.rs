use async_trait::async_trait;
use chrono::{NaiveTime, NaiveDate};
use serde::Serialize;
use std::ops::{Range, RangeInclusive};

use crate::data::{schedule::{self as regular, Subgroup}, Weekday};
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


// groups

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
    pub time: Option<Primitive<Option<Range<NaiveTime>>>>
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

// teachers

#[derive(Debug, Clone, Serialize)]
pub struct TchrSubject {
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num: Option<Primitive<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Changes<Subgroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabinet: Option<Primitive<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Primitive<Option<Range<NaiveTime>>>>
}
#[async_trait]
impl DetailedCmp<regular::TchrSubject, TchrSubject> for TchrSubject {
    async fn compare(
        old: Option<regular::TchrSubject>,
        new: Option<regular::TchrSubject>
    ) -> TchrSubject {
        let name = new.as_ref().map(|new| new.name.clone());
        let num = Primitive::new(
            old.as_ref().map(|old| old.num),
            new.as_ref().map(|new| new.num),
        );
        let groups = Changes::compare(
            old.as_ref().map(|old| old.groups.clone()),
            new.as_ref().map(|new| new.groups.clone()),
        ).await;
        let cabinet = Primitive::new(
            old.as_ref().map(|old| old.cabinet.clone()),
            new.as_ref().map(|new| new.cabinet.clone()),
        );
        let time = Primitive::new(
            old.as_ref().map(|old| old.time.clone()),
            new.as_ref().map(|new| new.time.clone()),
        );

        TchrSubject {
            name,
            num: if num.is_different_hash() {
                Some(num)
            } else {
                None
            },
            groups: if groups.any_changes() {
                Some(groups)
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
pub struct TchrDay {
    pub weekday: Option<Weekday>,
    pub subjects: DetailedChanges<regular::TchrSubject, TchrSubject>
}
#[async_trait]
impl DetailedCmp<regular::TchrDay, TchrDay> for TchrDay {
    async fn compare(
        old: Option<regular::TchrDay>,
        new: Option<regular::TchrDay>
    ) -> TchrDay {
        let weekday = new.as_ref().map(|new| new.weekday.clone());
        let subjects = DetailedChanges::compare(
            old.map(|old| old.subjects.clone()),
            new.map(|new| new.subjects.clone()),
        ).await;

        TchrDay { weekday, subjects }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TchrTeacher {
    pub name: Option<String>,
    pub days: DetailedChanges<regular::TchrDay, TchrDay>
}
#[async_trait]
impl DetailedCmp<regular::TchrTeacher, TchrTeacher> for TchrTeacher {
    async fn compare(
        old: Option<regular::TchrTeacher>,
        new: Option<regular::TchrTeacher>
    ) -> TchrTeacher {
        let name = new.as_ref().map(|new| new.name.clone());
        let days = DetailedChanges::compare(
            old.map(|old| old.days.clone()),
            new.map(|new| new.days.clone()),
        ).await;

        TchrTeacher { name, days }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TchrPage {
    pub raw: Option<String>,
    pub date: Primitive<RangeInclusive<NaiveDate>>,
    pub teachers: DetailedChanges<regular::TchrTeacher, TchrTeacher>
}
#[async_trait]
impl DetailedCmp<regular::TchrPage, TchrPage> for TchrPage {
    async fn compare(
        old: Option<regular::TchrPage>,
        new: Option<regular::TchrPage>
    ) -> TchrPage {
        let raw = new.as_ref().map(|new| new.raw.clone());
        let date = Primitive::new(
            old.as_ref().map(|old| old.date.clone()),
            new.as_ref().map(|new| new.date.clone()),
        );
        let teachers = DetailedChanges::compare(
            old.as_ref().map(|old| old.teachers.clone()),
            new.as_ref().map(|new| new.teachers.clone()),
        ).await;

        TchrPage { raw, date, teachers }
    }
}
