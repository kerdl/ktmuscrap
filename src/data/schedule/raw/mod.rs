mod index;
mod last;
pub mod table;
pub mod ignored;
pub mod fulltime;
pub mod remote;
pub mod error;

pub use index::Index;
pub use last::Last;

use std::ops::Range;
use chrono::NaiveTime;
use derive_new::new;
use serde_derive::{Serialize, Deserialize};
use strum_macros::{EnumString, Display};


#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    EnumString,
    Hash
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Type {
    FtDaily,
    FtWeekly,
    RWeekly,
    TchrFtDaily,
    TchrFtWeekly,
    TchrRWeekly
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    EnumString,
    Hash
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Groups,
    Teachers
}

#[derive(new, Debug, Clone)]
pub struct NumTime {
    pub num: u32,
    pub time: Option<Range<NaiveTime>>,
}