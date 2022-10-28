pub mod html;
pub mod table;

use log::info;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    data::schedule::raw::Zip, 
    SyncResult
};
use super::node;


pub async fn parse(schedule: Arc<RwLock<Zip>>) -> SyncResult<()> {
    todo!();

    /* 
    let schedule = schedule.read().await;

    let mut html_container = schedule.to_html_container().await?;

    let mut latest = html_container.latest().await;
    info!("latest: {}", latest.as_ref().unwrap().0);

    latest.unwrap().1.table();

    return Ok(());

    //let time_row = latest.as_mut().unwrap().1.time_table();
    //info!("time: {:?}", time_row.unwrap());

    
    Ok(())
    */
}