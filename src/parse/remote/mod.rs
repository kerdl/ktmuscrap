pub mod html;
pub mod table;
pub mod mapping;

use log::info;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    REMOTE_INDEX,
    data::schedule::{raw, Page}, 
    SyncResult,
    perf
};
use super::{node, error};


pub async fn parse(schedule: Arc<raw::Schedule>) -> SyncResult<()> {
    let zip = schedule.zip.read().await;

    let mut html_container = zip.to_remote_html_container().await?;

    if let Some(removed_paths) = html_container.clear_old().await {

        if removed_paths.len() > 0 {
            let index = REMOTE_INDEX.get().unwrap();
            index.ignored.write().await.extend(removed_paths.clone());

            index.save().await;
        }

    }

    let index = REMOTE_INDEX.get().unwrap();
    index.remove_ignored().await;


    let mut latest = html_container.latest().await;
    if latest.is_none() {
        return Err(error::NoLatestRemote.into())
    }
    let latest = latest.as_mut().unwrap();


    tokio::task::spawn_blocking(move || -> SyncResult<()> {
        Ok(())
    });

    let table = latest.1.table();
    if table.is_none() {
        return Err(error::NoTables::new(
            raw::Type::RWeekly
        ).into())
    }
    let table = table.unwrap();


    let mapping = table.mapping();
    if mapping.is_none() {
        return Err(error::NoMappings::new(
            raw::Type::RWeekly
        ).into())
    }
    let mapping = mapping.unwrap();


    mapping.page();


    *schedule.parsed.write().await = {
        mapping.page.take().map(|page| Arc::new(page))
    };


    Ok(())
}