pub mod html;
pub mod table;

use log::info;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    REMOTE_SCHEDULE_INDEX,
    data::schedule::raw::Zip, 
    SyncResult, 
    fs,
};
use super::node;


pub async fn parse(schedule: Arc<RwLock<Zip>>) -> SyncResult<()> {
    let schedule = schedule.read().await;

    let mut html_container = schedule.to_html_container().await?;

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
    info!("latest: {}", latest.as_ref().unwrap().0);

    let table = latest.as_mut().unwrap().1.table().unwrap();

    table.dick();

    //let time_row = latest.as_mut().unwrap().1.time_table();
    //info!("time: {:?}", time_row.unwrap());

    
    Ok(())
}