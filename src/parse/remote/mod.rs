pub mod html;
pub mod table;
pub mod mapping;

use log::info;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    REMOTE_SCHEDULE_INDEX,
    data::schedule::{raw, Page}, 
    SyncResult,
    perf
};
use super::{node, error};


pub async fn parse(schedule: Arc<RwLock<raw::Zip>>) -> SyncResult<Page> {
    let schedule = schedule.read().await;

    let mut html_container = schedule.to_remote_html_container().await?;

    if let Some(removed_paths) = html_container.clear_old().await {

        if removed_paths.len() > 0 {
            let mut index = REMOTE_SCHEDULE_INDEX.write().await;
            index.ignored.extend(removed_paths.clone());

            tokio::task::spawn_blocking(move || {
                let index = REMOTE_SCHEDULE_INDEX.blocking_read();
                index.save().unwrap()
            });
        }

    }

    let index = REMOTE_SCHEDULE_INDEX.read().await;
    index.remove_ignored().await;

    let mut latest = html_container.latest().await;

    let table = latest.as_mut().unwrap().1.table().unwrap();

    let mapping = table.mapping();
    if mapping.is_none() {
        return Err(error::NoMappings::new(raw::Type::RWeekly).into())
    }
    let mapping = mapping.unwrap();

    mapping.page();

    Ok(mapping.page.take().unwrap())
}