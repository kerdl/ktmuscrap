mod index;
pub mod html;
pub mod table;

pub use index::Index;

use std::path::PathBuf;

use crate::{
    SyncResult,
    fs
};


pub async fn latest(dir: &PathBuf) -> SyncResult<Option<PathBuf>> {
    let paths = fs::collect::file_paths_by_extension(dir, "html").await?;

    let mut container = html::Container::from_paths(paths).await?;

    let path = container.latest_path().await.map(|path_date| path_date.0);

    Ok(path)
}