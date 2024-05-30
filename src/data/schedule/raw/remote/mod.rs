pub mod html;
pub mod table;

use std::{path::PathBuf, collections::HashSet};

use log::debug;

use crate::{
    SyncResult,
    fs,
    schedule::{raw, update}
};


pub async fn latest(files: &Vec<update::File>, mode: raw::Mode) -> SyncResult<HashSet<PathBuf>> {
    let mut container = match mode {
        raw::Mode::Groups => html::Container::from_files(files.clone(), vec![]).await.unwrap(),
        raw::Mode::Teachers => html::Container::from_files(vec![], files.clone()).await.unwrap(),
    };

    let path = container.latest_paths(mode).await.into_iter()
        .map(|path_date| path_date.0)
        .collect::<HashSet<PathBuf>>();

    Ok(path)
}
