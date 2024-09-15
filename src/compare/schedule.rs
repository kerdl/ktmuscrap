use chrono::NaiveDate;
use serde::Serialize;
use std::ops::RangeInclusive;

use crate::data::schedule as regular;
use crate::compare::{DetailedChanges, Primitive, DetailedCmp};


#[derive(Debug, Clone, Serialize)]
pub struct Cabinet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<Primitive<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opposite: Option<Primitive<String>>
}
impl DetailedCmp<regular::Cabinet, Cabinet> for Cabinet {
    async fn compare(
        old: Option<regular::Cabinet>,
        new: Option<regular::Cabinet>
    ) -> Self {
        let primary = Primitive::new(
            old.as_ref().map(|old| old.primary.as_ref().map(|val| val.clone())).flatten(),
            new.as_ref().map(|new| new.primary.as_ref().map(|val| val.clone())).flatten()
        );
        let opposite = Primitive::new(
            old.as_ref().map(|old| old.primary.as_ref().map(|val| val.clone())).flatten(),
            new.as_ref().map(|new| new.primary.as_ref().map(|val| val.clone())).flatten()
        );

        Self {
            primary: if primary.is_different_hash() {
                Some(primary)
            } else {
                None
            },
            opposite: if opposite.is_different_hash() {
                Some(opposite)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Attender {
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabinet: Option<Primitive<regular::Cabinet>>
}
impl DetailedCmp<regular::Attender, Attender> for Attender {
    async fn compare(
        old: Option<regular::Attender>,
        new: Option<regular::Attender>
    ) -> Self {
        let name = if let Some(new) = &new {
            Some(new.name.clone())
        } else if let Some(old) = &old {
            Some(old.name.clone())
        } else {
            None
        };
        let cabinet = Primitive::new(
            old.as_ref().map(|old| old.cabinet.clone()),
            new.as_ref().map(|new| new.cabinet.clone())
        );

        Self {
            name,
            cabinet: if cabinet.is_different_hash() {
                Some(cabinet)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Subject {
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num: Option<Primitive<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenders: Option<DetailedChanges<regular::Attender, Attender>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabinet: Option<Primitive<regular::Cabinet>>
}
impl DetailedCmp<regular::Subject, Subject> for Subject {
    async fn compare(
        old: Option<regular::Subject>,
        new: Option<regular::Subject>
    ) -> Self {
        let name = if let Some(new) = &new {
            Some(new.name.clone())
        } else if let Some(old) = &old {
            Some(old.name.clone())
        } else {
            None
        };
        let num = Primitive::new(
            old.as_ref().map(|old| old.num),
            new.as_ref().map(|new| new.num),
        );
        let attenders = DetailedChanges::compare(
            old.as_ref().map(|old| old.attenders.clone()),
            new.as_ref().map(|new| new.attenders.clone()),
        ).await;
        let cabinet = Primitive::new(
            old.as_ref().map(|old| old.cabinet.clone()),
            new.as_ref().map(|new| new.cabinet.clone())
        );

        Self {
            name,
            num: if num.is_different_hash() {
                Some(num)
            } else {
                None
            },
            attenders: if attenders.has_changes() {
                Some(attenders)
            } else {
                None
            },
            cabinet: if cabinet.is_different_hash() {
                Some(cabinet)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Day {
    pub date: Option<NaiveDate>,
    pub subjects: DetailedChanges<regular::Subject, Subject>
}
impl DetailedCmp<regular::Day, Day> for Day {
    async fn compare(
        old: Option<regular::Day>,
        new: Option<regular::Day>
    ) -> Self {
        let date = if let Some(new) = &new {
            Some(new.date.clone())
        } else if let Some(old) = &old {
            Some(old.date.clone())
        } else {
            None
        };
        let subjects = DetailedChanges::compare(
            old.map(|old| old.subjects.clone()),
            new.map(|new| new.subjects.clone()),
        ).await;

        Self { date, subjects }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Mapping {
    pub name: Option<String>,
    pub days: DetailedChanges<regular::Day, Day>
}
impl DetailedCmp<regular::Formation, Mapping> for Mapping {
    async fn compare(
        old: Option<regular::Formation>,
        new: Option<regular::Formation>
    ) -> Self {
        let name = if let Some(new) = &new {
            Some(new.name.clone())
        } else if let Some(old) = &old {
            Some(old.name.clone())
        } else {
            None
        };
        let days = DetailedChanges::compare(
            old.map(|old| old.days.clone()),
            new.map(|new| new.days.clone()),
        ).await;

        Self { name, days }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub date: Primitive<RangeInclusive<NaiveDate>>,
    pub mappings: DetailedChanges<regular::Formation, Mapping>
}
impl DetailedCmp<regular::Page, Page> for Page {
    async fn compare(
        old: Option<regular::Page>,
        new: Option<regular::Page>
    ) -> Self {
        let date = Primitive::new(
            old.as_ref().map(|old| old.date.clone()),
            new.as_ref().map(|new| new.date.clone()),
        );
        let mappings = DetailedChanges::compare(
            old.as_ref().map(|old| old.formations.clone()),
            new.as_ref().map(|new| new.formations.clone()),
        ).await;

        Self { date, mappings }
    }
}
