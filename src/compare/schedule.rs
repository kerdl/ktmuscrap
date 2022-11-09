use async_trait::async_trait;
use chrono::NaiveTime;
use serde::Serialize;
use std::ops::Range;

use crate::data::schedule as regular;
use super::{Changes, DetailedChanges, Primitive, DetailedCmp};


#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    pub compared: Primitive<regular::Subject>,

    pub teachers: Changes<String>,
    pub cabinet: Primitive<Option<String>>,
    pub time: Primitive<Range<NaiveTime>>
}
#[async_trait]
impl DetailedCmp<regular::Subject, Subject> for Subject {
    async fn compare(
        old: Option<regular::Subject>,
        new: regular::Subject
    ) -> Subject {

        let compared = Primitive::new(old.clone(), new.clone());

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
#[async_trait]
impl DetailedCmp<regular::Day, Day> for Day {
    async fn compare(
        old: Option<regular::Day>,
        new: regular::Day
    ) -> Day {

        let compared = Primitive::new(old.clone(), new.clone());
        let subjects = DetailedChanges::compare(
            old.map(|old| old.subjects.clone()),
            new.subjects.clone()
        ).await;

        Day { compared, subjects }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub compared: Primitive<regular::Group>,

    pub days: DetailedChanges<regular::Day, Day>
}
#[async_trait]
impl DetailedCmp<regular::Group, Group> for Group {
    async fn compare(
        old: Option<regular::Group>,
        new: regular::Group
    ) -> Group {

        let compared = Primitive::new(old.clone(), new.clone());
        let days = DetailedChanges::compare(
            old.map(|old| old.days.clone()),
            new.days.clone()
        ).await;

        Group { compared, days }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub compared: Primitive<regular::Page>,

    pub groups: DetailedChanges<regular::Group, Group>
}
#[async_trait]
impl DetailedCmp<regular::Page, Page> for Page {
    async fn compare(
        old: Option<regular::Page>,
        new: regular::Page
    ) -> Page {

        let compared = Primitive::new(old.clone(), new.clone());
        let groups = DetailedChanges::compare(
            old.map(|old| old.groups.clone()),
            new.groups.clone()
        ).await;

        Page { compared, groups }
    }
}