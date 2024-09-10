pub mod sheet;
pub mod node;
pub mod date;
pub mod group;
pub mod teacher;
pub mod vacancy;
pub mod subject;
pub mod num;
pub mod cabinet;
pub mod error;

use std::{path::PathBuf, sync::Arc};
use crate::data::schedule::{raw::Kind, Last};
use crate::lifetime;


pub async fn dom_from_string(string: &String) -> Result<html_parser::Dom, html_parser::Error> {
    let wrapped_dom = unsafe {
        lifetime::extend(lifetime::Wrap(string))
    };
    let dom = tokio::task::spawn_blocking(move || -> Result<html_parser::Dom, html_parser::Error> {
        html_parser::Dom::parse(&wrapped_dom.0)
    }).await.unwrap()?;

    return Ok(dom)
}

async fn generic(
    paths: &[PathBuf],
    last: Arc<Last>,
    kind: Kind,
) -> std::io::Result<()> {
    sheet::from_paths(paths, kind).await
}

pub async fn groups(
    paths: &[PathBuf],
    last: Arc<Last>
) -> std::io::Result<()> {
    generic(paths, last, Kind::Groups).await
}

pub async fn teachers(
    paths: &[PathBuf],
    last: Arc<Last>
) -> std::io::Result<()> {
    generic(paths, last, Kind::Teachers).await
}