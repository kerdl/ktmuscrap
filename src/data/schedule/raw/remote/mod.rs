pub mod html;
pub mod table;

use std::{path::PathBuf, collections::HashSet};

use log::debug;

use crate::{
    SyncResult,
    fs,
    schedule::raw
};


pub async fn latest(dir: &PathBuf, mode: raw::Mode) -> SyncResult<HashSet<PathBuf>> {
    let paths = fs::collect::file_paths_by_extension(
        dir,
        "html"
    ).await.unwrap();

    let mut container = match mode {
        raw::Mode::Groups => html::Container::from_paths(paths, vec![]).await.unwrap(),
        raw::Mode::Teachers => html::Container::from_paths(vec![], paths).await.unwrap(),
    };

    let path = container.latest_paths(mode).await.into_iter()
        .map(|path_date| path_date.0)
        .collect::<HashSet<PathBuf>>();

    Ok(path)
}
