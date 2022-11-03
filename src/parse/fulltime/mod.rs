pub mod html;
pub mod tables;
pub mod mappings;

use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    REMOTE_SCHEDULE_INDEX,
    data::schedule::{raw}, 
    SyncResult,
    perf
};


pub async fn generic_parse(schedule: Arc<RwLock<raw::Zip>>, sc_type: raw::Type) -> SyncResult<()> {
    let schedule = schedule.read().await;

    let mut parser = schedule.to_fulltime_parser(sc_type).await?;

    let tables = parser.tables().unwrap();
    let mappings = tables.mappings().unwrap();
    let page = mappings.page();

    Ok(())
}

pub async fn parse_ft_weekly(schedule: Arc<RwLock<raw::Zip>>) -> SyncResult<()> {
    let sc_type = raw::Type::FtWeekly;

    generic_parse(schedule, sc_type).await
}

pub async fn parse_ft_daily(schedule: Arc<RwLock<raw::Zip>>) -> SyncResult<()> {
    let sc_type = raw::Type::FtDaily;

    generic_parse(schedule, sc_type).await
}