pub mod index;
pub mod table;
pub mod error;

pub use index::Index;

use serde_derive::{Serialize, Deserialize};
use strum_macros::{EnumString, Display};
use crate::data::schedule::attender;


/// # Kind of schedule
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    EnumString,
    Hash,
    Copy
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Groups,
    Teachers
}
impl Kind {
    pub fn as_attender(&self) -> attender::Kind {
        match self {
            Self::Groups => attender::Kind::Group,
            Self::Teachers => attender::Kind::Teacher
        }
    }
}

/// # Format of a lesson
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq,
    Eq,
    Hash,
    Copy
)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Fulltime,
    Remote,
    Unknown
}
