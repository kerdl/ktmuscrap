pub mod html;
pub mod table;
pub mod mapping;

use std::sync::Arc;

use crate::{
    REMOTE_INDEX,
    data::schedule::raw, 
    SyncResult,
    perf,
    data::json::SavingLoading,
};
use super::{node, error};


pub async fn parse(schedule: Arc<raw::ScheduleContainer>) -> SyncResult<()> {
    let zip = schedule.zip.read().await;

    let mut html_container = zip.to_remote_html_container().await?;

    if let Some(removed_paths) = html_container.clear_old().await {

        if removed_paths.len() > 0 {
            let index = REMOTE_INDEX.get().unwrap();
            index.ignored.write().await.extend(removed_paths.clone());

            index.clone().poll_save();
        }

    }

    let index = REMOTE_INDEX.get().unwrap();
    index.remove_ignored().await;


    let mut latest = html_container.latest().await;
    if latest.is_none() {
        return Err(error::NoLatestRemote.into())
    }
    let latest = latest.as_mut().unwrap();

    let latest_parser = latest.1.clone();


    let mut mapping = tokio::task::spawn_blocking(move || -> SyncResult<mapping::Parser> {

        let mut latest_parser = latest_parser;

        perf!(let table = latest_parser.table());
        if table.is_none() {
            return Err(error::NoTables::new(
                raw::Type::RWeekly
            ).into())
        }
        let table = table.unwrap();


        perf!(let mapping = table.mapping());
        if mapping.is_none() {
            return Err(error::NoMappings::new(
                raw::Type::RWeekly
            ).into())
        }
        let mapping = mapping.unwrap();

        perf!(let _ = mapping.page());


        Ok(table.take_mapping().unwrap())

    }).await??;

    
    *schedule.parsed.write().await = {
        mapping.page.take().map(|page| Arc::new(page))
    };


    Ok(())
}