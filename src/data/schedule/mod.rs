mod last;
mod notify;
mod interactor;
pub mod update;
pub mod raw;
pub mod debug;

pub use last::Last;
pub use notify::Notify;
pub use interactor::{Interactor, Lifetime};

use lazy_static::lazy_static;
use ngrammatic::{CorpusBuilder, Corpus, Pad};
use serde_derive::{Serialize, Deserialize};
use chrono::{NaiveDate, NaiveTime};
use strum_macros::{EnumString, Display};
use derivative::Derivative;
use std::ops::{Range, RangeInclusive};
use std::collections::HashMap;

use super::Weekday;


lazy_static! {
    pub static ref FULLTIME_WINDOW_CORPUS: Corpus = {
        let mut corpus = CorpusBuilder::new()
            .arity(2)
            .pad_full(Pad::Auto)
            .finish();
        corpus.add_text("Очные занятия");
        corpus
    };
    pub static ref ONLINE_IDENTIFIER_CORPUS: Corpus = {
        let mut corpus = CorpusBuilder::new()
            .arity(2)
            .pad_full(Pad::Auto)
            .finish();
        corpus.add_text("Онлайн");
        corpus
    };
}

/// # Format of a lesson
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq,
    Eq,
    Hash
)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    /// Means you take ur ass
    /// and go to this fucking
    /// retarded place I fucking
    /// hate it 3 hours spent in
    /// the road so much retarded
    /// subjects and people there
    /// being a paid design 
    /// college student is the worst 
    /// fucking time of my life 
    /// aaaaaaaaaaaaaaaaaa
    /// (i'm so happy i have dropped out)
    Fulltime,
    /// Means you open up
    /// a `Zoom Meetings` computer
    /// program designed for
    /// remote meetings over
    /// `Internet®` connection
    /// and paste the meeting
    /// details into it
    Remote
}

/// # Schedule type
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone, 
    PartialEq, 
    Eq,
    EnumString,
    Display,
    Hash
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Type {
    /// Was parsed from a weekly
    /// (`ft_weekly` and `r_weekly`) schedule
    Weekly,
    /// Was parsed from a daily
    /// (`ft_daily` or `r_weekly`) schedule
    Daily
}

// groups

/// # Single subject (lesson) in a `Day`
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone
)]
pub struct Subject {
    /// # Raw representation, before parsing
    /// 
    /// ## Examples
    /// - **"ебанохулогия Ебанько Х.Й., ауд.69"**
    /// (*"fuckindicklogy Eban'ko D.C., cab.69"*)
    /// - **"Ебанохулогия. Ебанько Х.Й."**
    /// (*"Fuckindicklogy. Eban'ko D.C."*)
    /// 
    /// !!! (**D.C.** is **DICK**) !!!
    #[derivative(Hash="ignore")]
    pub raw: String,
    /// # Subject number
    pub num: u32,
    /// # Subject time range
    /// 
    /// - parsed from smth like
    /// `9:00-10:00`
    pub time: Range<NaiveTime>,
    /// # Subject name
    /// 
    /// - doesn't have a specific format,
    /// so comparing two same subjects 
    /// that are written different would
    /// be difficult
    /// 
    /// ## Examples
    /// - `Fulltime`s are usually written like
    ///     - **"ебанохулогия"** (*"fuckindicklogy"*)
    /// - `Remote`s are always written like
    ///     - **"Eбанохулогия."** (*"Fuckindicklogy."*)
    pub name: String,
    /// # Subject format
    /// 
    /// - see more in an enum itself
    pub format: Format,
    /// # Subject teachers
    /// 
    /// - yes, we have multiple teachers
    /// for a signle subject
    /// 
    /// ## Examples
    /// - **"`Ебанько Х.Й.`"** (*"Eban'ko D.C."*)
    /// - `Ебанько Х.` (*"Eban'ko D."*)
    /// 
    /// !!! (**D.C.** is **DICK**) !!!
    pub teachers: Vec<String>,
    /// # Subject's cabinet
    /// 
    /// - only makes sense if subject's
    /// format is `Fulltime`
    /// 
    /// ## Examples
    /// - **"ауд.69"** (*"cab.69"*)
    /// - **"ауд.69б"** (*"cab.69b"*)
    /// - **"ауд.69,78"** (*"cab.69,78"*)
    /// - ...
    pub cabinet: Option<String>,
}
impl Subject {
    pub fn is_fulltime_window(&self) -> bool {
        let is_similar_to_fulltime_window = {
            FULLTIME_WINDOW_CORPUS
                .search(&self.name, 0.5)
                .first()
                .is_some()
        };
        let no_teachers = self.teachers.is_empty();

        is_similar_to_fulltime_window && no_teachers
    }

    pub fn is_unknown_window(&self) -> bool {
        let no_teachers = self.teachers.is_empty();

        !self.is_fulltime_window() && no_teachers
    }
}
impl PartialEq for Subject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// # Single weekday (Mon, Tue) in a week
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone
)]
pub struct Day {
    /// # Raw name of a weekday
    /// ## Examples
    /// - **"Понедельник"** (*"Monday"*)
    /// - **"вторник"** (*"tuesday"*)
    /// - ...
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub weekday: Weekday,
    /// # Its date
    /// 
    /// - `year`, `month` and `day`
    /// - `time` is useless here
    pub date: NaiveDate,
    /// # List of subjects on this day
    pub subjects: Vec<Subject>,
}
impl PartialEq for Day {
    fn eq(&self, other: &Self) -> bool {
        self.weekday == other.weekday
    }
}

/// # Group's full schedule container
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone
)]
pub struct Group {
    /// # Raw header of a group
    /// 
    /// - each group has its label
    /// in the big official schedule
    /// 
    /// ## Examples
    /// - for `Fulltime`: 
    ///     - **"РАСПИСАНИЕ ГРУППЫ 1КДД69 неделя C 01/01/69 по 07/01/69"**
    ///     (*"SCHEDULE FOR GROUP 1KDD69 week from 01/01/69 to 07/01/69"*)
    /// - for `Remote`:
    ///     - **"1-кДД-69"**
    ///     (*"1-kDD-69"*)
    #[derivative(Hash="ignore")]
    pub raw: String,
    /// # Friendly group name
    /// 
    /// - taken and prettified from 
    /// a `raw` field, should always 
    /// be in the same format
    /// 
    /// ## Example
    /// - **"1КДД69"** (*"1KDD69"*)
    pub name: String,
    /// ## List of weekdays for this group
    pub days: Vec<Day>,
}
impl Group {
    pub fn remove_days_except(&mut self, date: &NaiveDate) {
        while let Some(index) = self.days.iter().position(
            |day| &day.date != date
        ) {
            self.days.remove(index);
        }
    }
}
impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// # Whole schedule page
/// - contains a list of groups
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    /// # Raw page header
    /// 
    /// - different for each 
    /// schedule type
    /// 
    /// ## Examples
    /// - `ft_weekly`: takes first group's header
    ///     - **"РАСПИСАНИЕ ГРУППЫ 1КДД69 неделя C 01/01/69 по 07/01/69"**
    ///     (*"SCHEDULE FOR GROUP 1KDD69 week from 01/01/69 to 07/01/69"*)
    /// 
    /// - `ft_daily`: usually has
    ///     - **"Очное расписание групп первого и третьего потоков на 01.01."**
    ///     (*"Fulltime schedule for groups of 1st and 3rd courses on 01.01."*)
    /// 
    /// but these idiots once
    /// somehow corrupted (cutted) it, 
    /// breaking bad the whole parser,
    /// so just for stability we'll
    /// only search for group headers
    /// 
    /// - `r_weekly`: we take the 1B table cell
    /// (too sad not 2B i wanted her to step on my dick)
    ///     - **"понедельник 01.01.69"**
    ///     (*"monday 01.01.69"*)
    pub raw: String,
    /// # Raw schedule types used to make this `Page`
    pub raw_types: Vec<raw::Type>,
    /// # What schedule type this `Page` is
    pub sc_type: Type,
    /// # The date this page relates to
    pub date: RangeInclusive<NaiveDate>,
    /// # List of groups on this page
    pub groups: Vec<Group>,
}
impl Page {
    pub fn remove_groups_except(&mut self, name: String) {
        while let Some(index) = self.groups.iter().position(
            |group| group.name != name
        ) {
            self.groups.remove(index);
        }
    }
}


// teachers

#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    Eq,
    PartialEq
)]
pub struct Subgroup {
    pub group: String,
    pub subgroup: Option<String>,
}

/// # Single subject (lesson) in a `Day`
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone
)]
pub struct TchrSubject {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub num: u32,
    pub time: Range<NaiveTime>,
    pub name: String,
    pub format: Format,
    pub groups: Vec<Subgroup>,
    pub cabinet: Option<String>,
}
impl PartialEq for TchrSubject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// # Single weekday (Mon, Tue) in a week
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone
)]
pub struct TchrDay {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub weekday: Weekday,
    pub date: NaiveDate,
    pub subjects: Vec<TchrSubject>,
}
impl PartialEq for TchrDay {
    fn eq(&self, other: &Self) -> bool {
        self.weekday == other.weekday
    }
}

/// # Group's full schedule container
#[derive(Derivative)]
#[derivative(Hash)]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone
)]
pub struct TchrTeacher {
    #[derivative(Hash="ignore")]
    pub raw: String,
    pub name: String,
    pub days: Vec<TchrDay>,
}
impl TchrTeacher {
    pub fn remove_days_except(&mut self, date: &NaiveDate) {
        while let Some(index) = self.days.iter().position(
            |day| &day.date != date
        ) {
            self.days.remove(index);
        }
    }
}
impl PartialEq for TchrTeacher {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TchrPage {
    pub raw: String,
    pub raw_types: Vec<raw::Type>,
    pub sc_type: Type,
    pub date: RangeInclusive<NaiveDate>,
    #[serde(skip)]
    pub num_time_mappings: Option<HashMap<u32, Range<NaiveTime>>>,
    pub teachers: Vec<TchrTeacher>,
}
impl TchrPage {
    pub fn remove_teachers_except(&mut self, name: String) {
        while let Some(index) = self.teachers.iter().position(
            |teacher| teacher.name != name
        ) {
            self.teachers.remove(index);
        }
    }
}