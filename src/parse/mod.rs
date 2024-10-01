pub mod sheet;
pub mod css;
pub mod node;
pub mod date;
pub mod group;
pub mod teacher;
pub mod attender;
pub mod subject;
pub mod cabinet;

use std::path::PathBuf;
use crate::data::schedule;
use crate::data::schedule::raw::Kind;
use crate::lifetime;


pub async fn dom_from_string(string: &String)
    -> Result<html_parser::Dom, html_parser::Error>
{
    let wrapped_dom = unsafe {
        lifetime::extend(lifetime::Wrap(string))
    };
    let dom = tokio::task::spawn_blocking(
        move || -> Result<html_parser::Dom, html_parser::Error>
    {
        html_parser::Dom::parse(&wrapped_dom.0)
    }).await.unwrap()?;

    return Ok(dom)
}

async fn generic(paths: &[PathBuf], kind: Kind)
    -> Vec<Result<schedule::Page, sheet::ParsingError>>
{
    sheet::from_paths(paths, kind).await
}

pub async fn groups(paths: &[PathBuf])
    -> Vec<Result<schedule::Page, sheet::ParsingError>>
{
    generic(paths, Kind::Groups).await
}

pub async fn teachers(paths: &[PathBuf])
    -> Vec<Result<schedule::Page, sheet::ParsingError>>
{
    generic(paths, Kind::Teachers).await
}