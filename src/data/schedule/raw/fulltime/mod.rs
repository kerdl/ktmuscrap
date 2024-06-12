pub mod html;
pub mod table;

use std::{path::PathBuf, ffi::OsStr};

use crate::{
    SyncResult,
    fs,
    schedule::update
};


pub async fn latest(files: &Vec<update::File>) -> Option<PathBuf> {
    files.iter().find(|file| file.path.extension() == Some(OsStr::new("html"))).map(|file| file.path.clone())
}