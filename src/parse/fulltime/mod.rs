pub mod html;
pub mod tables;
pub mod mappings;

use std::{sync::Arc, path::PathBuf};

use crate::{
    data::schedule::raw, 
    SyncResult,
    perf
};
use super::error;


#[derive(Debug)]
pub enum GenericParsingError {
    Loading(html::LoadingError),
    NoTables(error::NoTables),
    NoMappings(error::NoMappings)
}
impl From<html::LoadingError> for GenericParsingError {
    fn from(err: html::LoadingError) -> Self {
        Self::Loading(err)
    }
}
impl From<error::NoTables> for GenericParsingError {
    fn from(err: error::NoTables) -> Self {
        Self::NoTables(err)
    }
}
impl From<error::NoMappings> for GenericParsingError {
    fn from(err: error::NoMappings) -> Self {
        Self::NoMappings(err)
    }
}

async fn generic_parse(
    path: PathBuf,
    sc_type: raw::Type,
    last: Arc<raw::Last>,
) -> Result<(), GenericParsingError> {
    if ![raw::Type::FtDaily, raw::Type::FtWeekly].contains(&sc_type) {
        panic!("this parser only works with fulltime, you put {}", sc_type)
    }

    let parser = html::Parser::from_path(path, sc_type.clone()).await?;


    let sc_type_clone = sc_type.clone();

    let mut mappings = tokio::task::spawn_blocking(move || -> Result<mappings::Parser, GenericParsingError> {
        let mut parser = parser;

        // get all tables from html page
        perf!(let tables = parser.tables());
        if tables.is_none() {
            return Err(error::NoTables::new(sc_type_clone).into())
        }
        let tables = tables.unwrap();


        // map tables (connect each subject to time, num and weekday)
        perf!(let mappings = tables.mappings());
        if mappings.is_none() {
            return Err(error::NoMappings::new(sc_type_clone).into())
        }
        let mappings = mappings.unwrap();

        // generate page
        perf!(let _ = mappings.page());

        Ok(tables.take_mappings().unwrap())

    }).await.unwrap()?;

    let container = match sc_type {
        raw::Type::FtDaily => &last.ft_daily,
        raw::Type::FtWeekly => &last.ft_weekly,
        _ => unreachable!(),
    };

    *container.write().await = {
        mappings.page.take().map(|page| Arc::new(page))
    };
    
    Ok(())
}

pub async fn parse_ft_weekly(
    path: PathBuf,
    last: Arc<raw::Last>,
) -> Result<(), GenericParsingError> {
    let sc_type = raw::Type::FtWeekly;

    generic_parse(path, sc_type, last).await
}

pub async fn parse_ft_daily(
    path: PathBuf,
    last: Arc<raw::Last>,
) -> Result<(), GenericParsingError> {
    let sc_type = raw::Type::FtDaily;

    generic_parse(path, sc_type, last).await
}