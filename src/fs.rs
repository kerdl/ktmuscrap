use async_recursion::async_recursion;
use tokio::fs;
use std::path::PathBuf;

use crate::SyncResult;


#[async_recursion]
pub async fn collect_file_paths(dir_path: PathBuf) -> SyncResult<Vec<PathBuf>> {
    let mut paths = vec![];

    let mut entries = fs::read_dir(&dir_path).await?;

    while let Some(entry) = entries.next_entry().await? {

        if entry.path().is_dir() {
            // collect its files
            let folder_paths = collect_file_paths(entry.path()).await?;
            // add everything from that dir to our collection
            paths.extend(folder_paths);
        } else {
            paths.push(entry.path())
        }

    }

    Ok(paths)
}

pub async fn mass_file_remove(paths: Vec<PathBuf>) {
    for path in paths {
        tokio::spawn(async move {
            let _ = tokio::fs::remove_file(path).await;
        });
    }
}