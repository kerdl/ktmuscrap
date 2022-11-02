pub mod html;
pub mod tables;
pub mod mappings;

use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    REMOTE_SCHEDULE_INDEX,
    data::schedule::raw::Zip, 
    SyncResult,
    perf
};


pub async fn parse_ft_weekly(schedule: Arc<RwLock<Zip>>) -> SyncResult<()> {
    let schedule = schedule.read().await;

    let mut parser = schedule.to_fulltime_parser().await?;

    let tables = parser.tables().unwrap();
    let mappings = tables.mappings().unwrap();
    let page = mappings.page();

    Ok(())
}

pub async fn parse_ft_daily(schedule: Arc<RwLock<Zip>>) -> SyncResult<()> {
    let schedule = schedule.read().await;

    let mut parser = schedule.to_fulltime_parser().await?;

    let tables = parser.tables().unwrap();
    let mappings = tables.mappings().unwrap();
    let page = mappings.page();

    Ok(())
}