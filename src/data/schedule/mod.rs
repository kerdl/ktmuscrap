pub mod raw;
pub mod fulltime;
pub mod remote;
pub mod debug;
pub mod error;

use derive_new::new;
use lazy_static::lazy_static;
use log::warn;
use ngrammatic::{CorpusBuilder, Corpus, Pad};
use serde_derive::{Serialize, Deserialize};
use chrono::{NaiveDate, NaiveTime};
use strum_macros::{EnumString, Display};
use tokio::sync::RwLock;
use std::{ops::Range, cmp::Ordering, sync::Arc, path::PathBuf};

use crate::SyncResult;
use super::weekday::Weekday;


lazy_static! {
    static ref FULLTIME_WINDOW_CORPUS: Corpus = {
        let mut corpus = CorpusBuilder::new()
            .arity(2)
            .pad_full(Pad::Auto)
            .finish();
        
        corpus.add_text("Очные занятия");

        corpus
    };
}


/* CONTAINERS */

/// # Stores last converted schedule
/// - used for comparing schedules
#[derive(new, Clone, Debug)]
pub struct Last {
    path: PathBuf,
    pub weekly: Arc<RwLock<Option<Arc<Page>>>>,
    pub daily: Arc<RwLock<Option<Arc<Page>>>>
}
impl Last {
    fn from_serde_last(temp_last: SerdeLast) -> Last {
        Last::new(
            temp_last.path,
            Arc::new(RwLock::new(temp_last.weekly)),
            Arc::new(RwLock::new(temp_last.daily)),
        )
    }

    pub async fn save(&self) -> SyncResult<()> {
    
        let serde_last = Arc::new(SerdeLast::from_last(&self).await);

        tokio::spawn(async move {
            serde_last.save().await
        });

        Ok(())
    }

    pub fn load(path: PathBuf) -> SyncResult<Last> {

        let serde_last = SerdeLast::load(path)?;
        let last = Last::from_serde_last(serde_last);

        Ok(last)
    }

    pub async fn load_or_init(path: PathBuf) -> SyncResult<Last> {
        let last;

        if !path.exists() {
            last = Last::new(
                path, 
                Arc::new(RwLock::new(None)),
                Arc::new(RwLock::new(None))
            );
            last.save().await?;
        } else {
            last = Last::load(path)?;
        }

        Ok(last)
    }

    pub async fn clear_weekly(&self) -> SyncResult<()> {
        *self.weekly.write().await = None;
        self.save().await
    }

    pub async fn clear_daily(&self) -> SyncResult<()> {
        *self.daily.write().await = None;
        self.save().await
    }

    pub async fn clear_from_raw_type(&self, sc_type: &raw::Type) -> SyncResult<()> {
        match sc_type {
            raw::Type::FtDaily => {
                self.clear_daily().await
            }
            raw::Type::FtWeekly => {
                self.clear_weekly().await
            }
            raw::Type::RWeekly => {
                self.clear_daily().await?;
                self.clear_weekly().await
            }
        }
    }
}

#[derive(new, Serialize, Deserialize)]
struct SerdeLast {
    path: PathBuf,
    weekly: Option<Arc<Page>>,
    daily: Option<Arc<Page>>
}
impl SerdeLast {
    pub async fn from_last(last: &Last) -> SerdeLast {
        SerdeLast::new(
            last.path.clone(),
            last.weekly.read().await.as_ref().map(|page| page.clone()),
            last.daily.read().await.as_ref().map(|page| page.clone()),
        )
    }

    pub async fn save(self: Arc<Self>) -> SyncResult<()> {

        if let Err(err) = tokio::task::spawn_blocking(move || -> SyncResult<()> {

            let ser = serde_json::ser::to_string_pretty(&self)?;
            std::fs::write(&self.path.clone(), ser)?;

            Ok(())
        }).await? {
            warn!("error saving data::schedule::Last {:?}", err)
        }

        Ok(())
    }

    pub fn load(path: PathBuf) -> SyncResult<SerdeLast> {
        let de = std::fs::read_to_string(path)?;
        let last: SerdeLast = serde_json::de::from_str(&de)?;

        Ok(last)
    }
}

/* SCHEDULE */

/// # Format of a lesson
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq,
    Eq
)]
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

/// # Schedule type
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone, 
    PartialEq, 
    Eq,
    EnumString,
    Display
)]
pub enum Type {
    /// Was parsed from a weekly
    /// (`ft_weekly` and `r_weekly`) schedule
    Weekly,
    /// Was parsed from a daily
    /// (`ft_daily` or `r_weekly`) schedule
    Daily
}

/// # Single subject (lesson) in a `Day`
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    Eq
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
impl Ord for Subject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num.cmp(&other.num)
    }
}
impl PartialOrd for Subject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Subject {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

/// # Single weekday (Mon, Tue) in a week
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    Eq,
)]
pub struct Day {
    /// # Raw name of a weekday
    /// ## Examples
    /// - **"Понедельник"** (*"Monday"*)
    /// - **"Вторник"** (*"Tuesday"*)
    /// - ...
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
impl Ord for Day {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weekday.cmp(&other.weekday)
    }
}
impl PartialOrd for Day {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Day {
    fn eq(&self, other: &Self) -> bool {
        self.weekday == other.weekday
    }
}

/// # Group's full schedule container
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn remove_days_except(&mut self, date: NaiveDate) {
        while let Some(last) = self.days.iter().position(
            |day| day.date != date
        ) {
            self.days.remove(last);
        }
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
    pub date: Range<NaiveDate>,
    /// # List of groups on this page
    pub groups: Vec<Group>,
}