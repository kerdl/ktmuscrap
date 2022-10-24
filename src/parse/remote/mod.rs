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


pub async fn choose_newest(schedule: Arc<RwLock<Zip>>) -> DynResult<Bytes> {
    // path to schedule folder (./temp/<schedule_type>)
    let path_to_folder = schedule.read().await.path();

    // all paths of files inside given dir
    // and all its subdirectories
    let all_file_paths = fs::collect_file_paths(path_to_folder).await?;

    // { <path to schedule>: <declared date in that schedule>}
    let path_date_map: HashMap<PathBuf, NaiveDate> = HashMap::new();

    // vec of html files
    let html_paths = all_file_paths
        .into_iter()
        .filter(|path| {
            path.extension().is_some() // has extension
            && path.extension().unwrap() == "html" // and that extension is "html"
        })
        .collect::<Vec<PathBuf>>(); // create new vec of filtered files

    for html_path in html_paths {
        let html = tokio::fs::read(html_path).await?;

        let string = String::from_utf8(html)?;
    }

    Ok(Bytes::from(vec![]))
}

pub async fn parse(schedule: Arc<RwLock<Zip>>) {
    let newest_html = choose_newest(schedule.clone()).await;
}