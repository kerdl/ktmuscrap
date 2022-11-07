use chrono::NaiveDateTime;
use serde_derive::{Serialize, Deserialize};

use std::path::PathBuf;

use super::Schedule;


pub struct Index {
    pub updated: NaiveDateTime,
    pub types: Vec<Schedule>
}