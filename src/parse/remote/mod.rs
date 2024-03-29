pub mod html;
pub mod table;
pub mod mapping;

use std::{sync::Arc, path::PathBuf};

use crate::{
    data::schedule::raw, 
    SyncResult,
    perf,
};
use super::{node, error};


pub async fn parse(
    path: PathBuf,
    last: Arc<raw::Last>,
) -> SyncResult<()> {
    let parser = html::Parser::from_path(path).await?;

    let mut mapping = tokio::task::spawn_blocking(move || -> SyncResult<mapping::Parser> {
        let mut parser = parser;

        perf!(let table = parser.table());
        if table.is_none() {
            return Err(error::NoTables::new(
                raw::Type::RWeekly
            ).into())
        }
        let table = table.unwrap();

        perf!(let mapping = table.mapping_v2());
        if mapping.is_none() {
            return Err(error::NoMappings::new(
                raw::Type::RWeekly
            ).into())
        }
        let mapping = mapping.unwrap();

        perf!(let _ = mapping.page_v2());

        Ok(table.take_mapping().unwrap())
    }).await??;

    *last.r_weekly.write().await = {
        mapping.page.take().map(|page| Arc::new(page))
    };

    Ok(())
}