pub mod short;

use serde_derive::{Serialize, Deserialize};
use strum_macros::{EnumIter, Display};


#[derive(
    Serialize, 
    Deserialize, 
    Display, 
    EnumIter, 
    Debug, 
    Clone, 
    PartialEq, 
    Eq,
    PartialOrd,
    Ord,
    Hash
)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}
impl Weekday {
    pub fn as_short_str(&self) -> &'static str {
        match self {
            Self::Monday => short::MONDAY,
            Self::Tuesday => short::TUESDAY,
            Self::Wednesday => short::WEDNESDAY,
            Self::Thursday => short::THURSDAY,
            Self::Friday => short::FRIDAY,
            Self::Saturday => short::SATURDAY,
            Self::Sunday => short::SUNDAY
        }
    }

    pub fn from_short_str(string: &str) -> Option<Self> {
        match string {
            short::MONDAY => Some(Self::Monday),
            short::TUESDAY => Some(Self::Tuesday),
            short::WEDNESDAY => Some(Self::Wednesday),
            short::THURSDAY => Some(Self::Thursday),
            short::FRIDAY => Some(Self::Friday),
            short::SATURDAY => Some(Self::Saturday),
            short::SUNDAY => Some(Self::Sunday),
            _ => None
        }
    }

    pub fn as_chrono(&self) -> chrono::Weekday {
        match self {
            Self::Monday => chrono::Weekday::Mon,
            Self::Tuesday => chrono::Weekday::Tue,
            Self::Wednesday => chrono::Weekday::Wed,
            Self::Thursday => chrono::Weekday::Thu,
            Self::Friday => chrono::Weekday::Fri,
            Self::Saturday => chrono::Weekday::Sat,
            Self::Sunday => chrono::Weekday::Sun
        }
    }

    pub fn from_chrono(chrono: &chrono::Weekday) -> Self {
        match chrono {
            chrono::Weekday::Mon => Self::Monday,
            chrono::Weekday::Tue => Self::Tuesday,
            chrono::Weekday::Wed => Self::Wednesday,
            chrono::Weekday::Thu => Self::Thursday,
            chrono::Weekday::Fri => Self::Friday,
            chrono::Weekday::Sat => Self::Saturday,
            chrono::Weekday::Sun => Self::Sunday
        }
    }
}
