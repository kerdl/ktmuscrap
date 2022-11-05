pub mod html;
pub mod tables;
pub mod mappings;

use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    REMOTE_INDEX,
    data::schedule::{raw, Page}, 
    SyncResult,
    merge,
    perf
};
use super::error;


async fn generic_parse(
    schedule: Arc<raw::Schedule>, 
    sc_type: raw::Type
) -> SyncResult<()> {

    let zip = schedule.zip.read().await;

    // get html parser
    let parser = zip.to_fulltime_parser(sc_type.clone()).await?;

    let mut mappings = tokio::task::spawn_blocking(move || -> SyncResult<mappings::Parser> {

        let mut parser = parser;

        // get all tables from html page
        let tables = parser.tables();
        if tables.is_none() {
            return Err(error::NoTables::new(sc_type).into())
        }
        let tables = tables.unwrap();


        // map tables (connect each subject to time, num and weekday)
        let mappings = tables.mappings();
        if mappings.is_none() {
            return Err(error::NoMappings::new(sc_type).into())
        }
        let mappings = mappings.unwrap();

        // generate page
        mappings.page();

        Ok(mappings.clone())

    }).await??;

    *schedule.parsed.write().await = {
        mappings.page.take().map(|page| Arc::new(page))
    };
    
    Ok(())
}

pub async fn parse_ft_weekly(schedule: Arc<raw::Schedule>) -> SyncResult<()> {
    let sc_type = raw::Type::FtWeekly;

    generic_parse(schedule, sc_type).await
}

pub async fn parse_ft_daily(schedule: Arc<raw::Schedule>) -> SyncResult<()> {
    let sc_type = raw::Type::FtDaily;

    generic_parse(schedule, sc_type).await
}