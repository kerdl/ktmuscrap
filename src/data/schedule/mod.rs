mod file;
mod notify;
mod last;
pub mod raw;
pub mod attender;

pub use file::File;
pub use notify::Notify;
pub use last::Last;

use crate::{compare::FindingCmp, regexes};

use serde_derive::{Serialize, Deserialize};
use chrono::NaiveDate;
use derivative::Derivative;
use std::ops::RangeInclusive;


/// # A cabinet in `Attender`
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq
)]
pub struct Cabinet {
    /// # A value taken from the original schedule
    pub primary: Option<String>,
    /// # A value taken from the opposite schedule
    /// This is only used to shame the people
    /// responsible for maintaining the schedule
    /// if `primary` and `opposite` don't match.
    /// These differences are shown in the bot.
    /// 
    /// ## For example
    /// If this instance belongs to a group schedule,
    /// `opposite` would reference a cabinet found in
    /// teacher's schedule.
    pub opposite: Option<String>
}
impl FindingCmp for Cabinet {
    fn is_partially_same_with(&self, other: &Self) -> bool {
        self.primary == other.primary
    }
}
impl Default for Cabinet {
    fn default() -> Self {
        Self {
            primary: None,
            opposite: None
        }
    }
}
impl Cabinet {
    pub fn do_versions_match(&self) -> bool {
        self.primary == self.opposite
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.primary, &mut self.opposite);
    }
}

/// # Single attender (teacher/group) in a `Subject`
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq
)]
pub struct Attender {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub kind: attender::Kind,
    pub name: String,
    pub cabinet: Cabinet
}
impl FindingCmp for Attender {
    fn is_partially_same_with(&self, other: &Self) -> bool {
        self.kind == other.kind &&
        self.name == other.name
    }
}

/// # Single subject (lesson) in a `Day`
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq
)]
pub struct Subject {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub name: String,
    pub num: u32,
    pub format: raw::Format,
    pub attenders: Vec<Attender>,
    pub cabinet: Cabinet
}
impl FindingCmp for Subject {
    fn is_partially_same_with(&self, other: &Self) -> bool {
        self.name == other.name &&
        self.num == other.num &&
        self.format == other.format
    }
}
impl Subject {
    pub fn in_numbered_filler(&self) -> bool {
        if self.raw.len() != 1 {
            return false;
        }
        regexes().digit.is_match(&self.raw)
    }
}

/// # Single day in Mapping
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone
)]
pub struct Day {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub date: NaiveDate,
    pub subjects: Vec<Subject>,
}
impl FindingCmp for Day {
    fn is_partially_same_with(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

/// # Group or teacher mapping
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone
)]
pub struct Formation {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub name: String,
    pub days: Vec<Day>,
}
impl FindingCmp for Formation {
    fn is_partially_same_with(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// # Whole schedule page
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    pub kind: raw::Kind,
    pub date: RangeInclusive<NaiveDate>,
    pub formations: Vec<Formation>,
}
impl Page {
    pub fn remove_except(&mut self, name: &str) {
        while let Some(index) = self.formations.iter().position(
            |map| map.name != name
        ) {
            self.formations.remove(index);
        }
    }
}
