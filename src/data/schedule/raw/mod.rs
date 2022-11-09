mod index;
mod last;
pub mod table;
pub mod ignored;
pub mod fulltime;
pub mod remote;
pub mod error;

pub use index::Index;
pub use last::Last;

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
    RWeekly
}
