//! # Three-step sheet parsing
//! - `html`: converting HTML to a table
//! - `table`: mapping the table: connecting subjects
//! to groups/teachers and dates
//! - `mappings`: creating final `Page` objects from mappings

pub mod html;
pub mod table;
pub mod mappings;

use log::debug;
use std::path::PathBuf;
use crate::data::schedule::raw::Kind;
use crate::lifetime;


#[derive(thiserror::Error, Debug)]
#[error("parsing error")]
pub enum ParsingError {
    Html(html::ParsingError),
    Table,
    Mapping
}


pub async fn from_path(
    path: &PathBuf,
    kind: Kind,
) -> Result<(), ParsingError> {
    let mut html_processor = html::Parser::from_path(path).await.unwrap();
    let tables = html_processor.parse().await;
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