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

    let schedule = schedule.read().await;

    let mut html_container = schedule.to_html_container().await?;

    let mut latest = html_container.latest().await;
    info!("latest: {}", latest.as_ref().unwrap().0);

    let table_parser = latest.as_mut().unwrap().1.to_table_parser().unwrap();

    table_parser.dick();

    //let time_row = latest.as_mut().unwrap().1.time_table();
    //info!("time: {:?}", time_row.unwrap());

    
    Ok(())
}