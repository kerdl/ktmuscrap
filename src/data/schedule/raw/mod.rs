mod index;
mod zip;
pub mod container;
pub mod table;
pub mod fulltime;
pub mod remote;

pub use self::zip::Zip;
pub use container::{
    Container,
    Schedule as ScheduleContainer
};

use lazy_static::lazy_static;
use ::zip::ZipArchive;
use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use strum_macros::{EnumString, Display};
use sha2::{Sha256, Digest};
use hex;
use reqwest;
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{io::Cursor, path::PathBuf, sync::Arc};


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
    FtDaily,
    FtWeekly,
    RWeekly
}
