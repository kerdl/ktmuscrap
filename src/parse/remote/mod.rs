pub mod html;
pub mod table;
pub mod mapping;

use std::{sync::Arc, path::PathBuf};
use log::warn;

use crate::{
    data::schedule::{raw, TchrPage}, 
    SyncResult,
    merge,
    perf,
};
use super::{node, error};


pub async fn parse(
    path: PathBuf,
    last: Arc<raw::Last>,
) -> SyncResult<()> {
    let parser = html::Parser::from_path(path).await;

    if let Err(err) = parser {
        warn!("remote parser initialization error: {:?}", err);
        return Err(err);
    }

    let parser = parser.unwrap();

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

pub async fn tchr_parse(
    paths: &[PathBuf],
    last: Arc<raw::Last>,
) -> SyncResult<()> {
    let mut tasks = vec![];
    let mut pages = vec![];
    let parsers = html::TchrParser::from_paths(paths).await?;

    for parser in parsers {
        let task = tokio::task::spawn_blocking(move || -> SyncResult<Option<TchrPage>> {
            let mut parser = parser;
    
            perf!(let table = parser.table());
            if table.is_none() {
                return Err(error::NoTables::new(
                    raw::Type::RWeekly
                ).into())
            }
            let table = table.unwrap();
    
            perf!(let (mapping, numtime_mappings) = table.mapping());
            if mapping.is_none() {
                return Err(error::NoMappings::new(
                    raw::Type::RWeekly
                ).into())
            }
            let mapping = mapping.unwrap();
    
            perf!(let _ = mapping.page());

            let mut page = table.take_mapping().unwrap().page;
            page.as_mut().map(|page| page.num_time_mappings = numtime_mappings);
    
            Ok(page)
        });
        tasks.push(task);
    }

    for task in tasks {
        if let Ok(Ok(Some(page))) = task.await {
            pages.push(page)
        }
    }

    let final_page = merge::weekly::tchr_r_page(&mut pages).await.ok().flatten();

    *last.tchr_r_weekly.write().await = {
        final_page.map(|page| Arc::new(page))
    };

    Ok(())
}