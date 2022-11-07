mod index;
mod zip;
pub mod container;
pub mod table;

pub use index::{Index, MiddleIndex};
pub use self::zip::Zip;
pub use container::{Container, Schedule};

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
pub enum Type {
    FtWeekly,
    FtDaily,
    RWeekly
}
