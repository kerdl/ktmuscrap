pub mod error;
pub mod raw;
pub mod weekday;

use std::ops::Range;
use serde_derive::{Serialize, Deserialize};
use chrono::{NaiveDate, NaiveTime};



/* CONTAINERS */

/// ## Container for typed schedule
/// - contains schedule data
/// and tells what type it is:
/// `Weekly` or `Daily`
pub struct Typed { 
    pub sc_type: Type,
    pub schedule: Page
}
impl Typed {
    pub fn new(sc_type: Type, schedule: Page) -> Typed {
        Typed { sc_type, schedule }
    }
}

/// ## Stores last converted schedule
/// - used for comparing schedules
pub struct Last {
    pub weekly: Option<Typed>,
    pub daily: Option<Typed>
}
impl Last {
    pub fn new(weekly: Option<Typed>, daily: Option<Typed>) -> Last {
        Last { weekly, daily }
    }
}
impl Default for Last {
    fn default() -> Last {
        Last::new(None, None)
    }
}



/* SCHEDULE */

/// ## Format of a lesson
#[derive(Serialize, Deserialize, Debug, Clone)]
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

/// ## Schedule type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Was parsed from a weekly
    /// (`ft_weekly`, `r_weekly`) schedule
    Weekly,
    /// Was parsed from a daily
    /// (`ft_daily`) schedule
    Daily
}

/// ## Single subject (lesson) in a `Weekday`
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subject {
    /// ## Raw representation, before parsing
    /// 
    /// ### Examples
    /// - **"ебанохулогия Ебанько Х.Й., ауд.69"**
    /// (*"fuckindicklogy Eban'ko D.C., cab.69"*)
    /// - **"Ебанохулогия. Ебанько Х.Й."**
    /// (*"Fuckindicklogy. Eban'ko D.C."*)
    /// 
    /// !!! (**D.C.** is **DICK**) !!!
    pub raw: String,
    /// ## Subject number
    /// 
    /// - since we don't have more than 
    /// 255 lessons per day, `u8` is enough
    /// - if you forked it and love studying 
    /// (you fucking nerd), feel free 
    /// to change it to `u128`
    pub num: u8,
    /// ## Subject time range
    /// 
    /// - parsed from smth like
    /// `9:00-10:00`
    pub time: Range<NaiveTime>,
    /// ## Subject name
    /// 
    /// - doesn't have a specific format,
    /// so comparing two same subjects 
    /// that are written different would
    /// be difficult
    /// 
    /// ### Examples
    /// - `Fulltime`s are usually written like
    ///     - **"ебанохулогия"** (*"fuckindicklogy"*)
    /// - `Remote`s are always written like
    ///     - **"Eбанохулогия."** (*"Fuckindicklogy."*)
    pub name: String,
    /// ## Subject format
    /// 
    /// - see more in an enum itself
    pub format: Format,
    /// ## Subject teachers
    /// 
    /// - yes, we have multiple teachers
    /// for a signle subject
    /// 
    /// ### Examples
    /// - **"`Ебанько Х.Й.`"** (*"Eban'ko D.C."*)
    /// - `Ебанько Х.` (*"Eban'ko D."*)
    /// 
    /// !!! (**D.C.** is **DICK**) !!!
    pub teachers: Vec<String>,
    /// ## Subject's cabinet
    /// 
    /// - only makes sense if subject's
    /// format is `Fulltime`
    /// 
    /// ### Examples
    /// - **"ауд.69"** (*"cab.69"*)
    /// - **"ауд.69б"** (*"cab.69b"*)
    /// - **"ауд.69,78"** (*"cab.69,78"*)
    /// - ...
    pub cabinet: Option<String>,
}

/// ## Single weekday (Mon, Tue) in a week
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Weekday {
    /// ## Raw name of a weekday
    /// ### Examples
    /// - **"Понедельник"** (*"Monday"*)
    /// - **"Вторник"** (*"Tuesday"*)
    /// - ...
    pub raw: String,
    /// ## Its date
    /// 
    /// - `year`, `month` and `day`
    /// - `time` is useless here
    pub date: NaiveDate,
    /// ## List of subjects on this day
    pub subjects: Vec<Subject>,
}

/// ## Group's full schedule container
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Group {
    /// ## Raw header of a group
    /// 
    /// - each group has its label
    /// in the big official schedule
    /// 
    /// ### Examples
    /// - for `Fulltime`: 
    ///     - **"РАСПИСАНИЕ ГРУППЫ 1КДД69 неделя C 01/01/69 по 07/01/69"**
    ///     (*"SCHEDULE FOR GROUP 1KDD69 week from 01/01/69 to 07/01/69"*)
    /// - for `Remote`:
    ///     - **"1-кДД-69"**
    ///     (*"1-kDD-69"*)
    pub raw: String,
    /// ## Friendly group name
    /// 
    /// - taken and prettified from 
    /// a `raw` field, should always 
    /// be in the same format
    /// 
    /// ### Example
    /// - **"1КДД69"** (*"1KDD69"*)
    pub name: String,
    /// ## List of weekdays for this group
    pub weekdays: Vec<Weekday>,
}

/// ## Whole schedule page
/// - contains a list of groups
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Page {
    /// ## Raw page header
    /// 
    /// - different for each 
    /// schedule type
    /// 
    /// ### Examples
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
    /// still search for group headers
    /// in case the primary one isn't found
    /// 
    /// - `r_weekly`: we take the 1B table cell
    /// (too sad not 2B i wanted her to step on my dick)
    ///     - **"понедельник 01.01.69"**
    ///     (*"monday 01.01.69"*)
    pub raw: String,
    /// ## The date this page relates to
    /// 
    /// ### Examples
    /// - ``
    pub date: NaiveDate,
    /// ## List of groups on this page
    pub groups: Vec<Group>,
}