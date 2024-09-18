//! # Two-step sheet parsing
//! - `html`: converting HTML to a table
//! - `table`: mapping the table: connecting subjects
//!   to groups/teachers and dates, constructing a `Page`

pub mod html;
pub mod table;

use log::debug;
use std::path::PathBuf;
use crate::data::schedule::raw::Kind;
use crate::lifetime;


#[derive(thiserror::Error, Debug)]
#[error("parsing error")]
pub enum ParsingError {
    Html(html::ParsingError),
    Table(table::ParsingError),
}
impl From<html::ParsingError> for ParsingError {
    fn from(value: html::ParsingError) -> Self {
        Self::Html(value)
    }
}
impl From<table::ParsingError> for ParsingError {
    fn from(value: table::ParsingError) -> Self {
        Self::Table(value)
    }
}


pub async fn from_path(
    path: &PathBuf,
    kind: Kind,
) -> Result<(), ParsingError> {
    let html_processor = html::Parser::from_path(path).await?;
    let table = html_processor.parse().await?;
    let table_processor = table::Parser::from_schema(table, kind);
    let mappings = table_processor.parse().await?;
    debug!("{:?} parsed", path);
    Ok(())
}

pub async fn from_paths(
    paths: &[PathBuf],
    kind: Kind,
) -> std::io::Result<()> {
    let mut handles = vec![];

    for path in paths {
        let wrapped_path = unsafe {
            lifetime::extend(lifetime::Wrap(path))
        };
        let handle = tokio::spawn(async move {
            from_path(&wrapped_path.0, kind).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await.unwrap();
    }

    Ok(())
}