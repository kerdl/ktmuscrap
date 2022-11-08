mod index;
mod ignored;
mod last;
mod zip;
pub mod container;
pub mod table;
pub mod fulltime;
pub mod remote;

pub use index::Index;
pub use ignored::Ignored;
pub use last::Last;
pub use self::zip::Zip;

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
#[serde(rename_all = "snake_case" )]
pub enum Type {
    FtDaily,
    FtWeekly,
    RWeekly
}
