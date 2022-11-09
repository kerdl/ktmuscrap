pub mod html;
pub mod table;

use std::path::PathBuf;

use crate::{
    SyncResult,
    fs
};


pub async fn latest(dir: &PathBuf) -> tokio::io::Result<Option<PathBuf>> {
    if let Some(path) = fs::collect::file_paths(dir).await?.iter().find(
        |path| if let Some(ext) = path.extension() {
            ext == "html"
        } else {
            false
        }
    ) {
        return Ok(Some(path.to_owned()))
    }

    Ok(None)
}