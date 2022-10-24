use log::info;
use chrono::NaiveDate;
use html_parser::Dom;
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{sync::Arc, path::{Path, PathBuf, self}, collections::HashMap};

use crate::{data::schedule::raw::{Zip, Type}, fs, DynResult};


pub struct Html {
    dom: Dom
}
impl Html {
    pub fn from_string(string: &str) -> DynResult<Html> {
        Ok(Html {
            dom: Dom::parse(string)?
        })
    }

    /// ## Get base date this schedule is for
    pub fn base_date(&self) -> Option<NaiveDate> {
        todo!()
    }
}

pub async fn parse(schedule: Arc<RwLock<Zip>>) {

}