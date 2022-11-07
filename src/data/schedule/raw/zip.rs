use derive_new::new;
use zip::ZipArchive;
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{
    path::PathBuf,
    io::Cursor
};

use crate::{
    SyncResult,
    DynResult,
    REMOTE_INDEX,
    REMOTE_INDEX_PATH,
    parse::fulltime,
    fs,
};
use super::{
    Type as RawType,
    super::{
        error,
        remote,
    }
};


#[derive(new, Debug)]
pub struct Zip {
    sc_type: RawType,
    pub content: RwLock<Option<Bytes>>,
}
impl Zip {
    pub fn from_sc_type(sc_type: RawType) -> Zip {
        Zip::new(sc_type, RwLock::new(None))
    }

    pub async fn set_content(&self, content: Bytes) {
        let mut field = self.content.write().await;
        *field = Some(content);
    }

    pub async fn clear_content(&self) {
        let mut field = self.content.write().await;
        field.take();
    }

    /// ## Generate path for this schedule type's folder
    pub fn path(&self) -> PathBuf {
        // naming the directory the same as schedule type
        let dir_name = self.sc_type.to_string();
        // make relative path to this dir
        let dir_path = crate::TEMP_PATH.join(dir_name);

        dir_path
    }

    pub async fn create_folder(&self) -> tokio::io::Result<()> {
        tokio::fs::create_dir(self.path()).await
    }

    /// ## Remove folder in `self.path()`
    pub async fn remove_folder(&self) -> DynResult<()> {
        let path = self.path();
        tokio::fs::remove_dir_all(path).await?;
        
        Ok(())
    }

    pub async fn remove_folder_if_exists(&self) -> DynResult<()> {
        let path = self.path();

        if !path.exists() {
            return Ok(())
        }

        tokio::fs::remove_dir_all(path).await?;

        Ok(())
    }

    /// ## Remove folder and clear content loaded into RAM
    pub async fn delete(&self) -> DynResult<()> {
        self.remove_folder_if_exists().await?;
        self.clear_content().await;

        Ok(())
    }

    /// ## Extract content to `./temp/<schedule_type>`
    pub async fn extract(&self) -> DynResult<()> {
        let content = self.content.read().await;

        if content.is_none() {
            return Err(error::ExtractingEmptyContent.into())
        }

        let content = content.as_ref().unwrap().clone();
        let cursor = Cursor::new(&content[..]);

        let dir_path = self.path();

        // if directory for this schedule type doesn't exist
        if !dir_path.exists() {
            // create it
            self.create_folder().await?
        }

        // parse archive
        let mut archive = ZipArchive::new(cursor)?;

        // extract it to directory we just created
        archive.extract(&dir_path)?;

        Ok(())
    }

    /// ## Get all paths to HTML files in this ZIP
    /// - not only in root, but in all subdirectories
    pub async fn html_paths(&self) -> SyncResult<Vec<PathBuf>> {
        let mut all_file_paths = fs::collect_file_paths(self.path()).await?;

        if all_file_paths.contains(&REMOTE_INDEX_PATH) {

            for ignored_path in {
                REMOTE_INDEX.get().unwrap()
                .ignored.read().await.iter()
            } {

                if let Some(index) = all_file_paths.iter().position(
                    |path| path == ignored_path
                ) {
                    all_file_paths.remove(index);
                }
            }
        }

        let filtered_htmls = all_file_paths
            .into_iter()
            .filter(|path: &PathBuf| {
                path.extension().is_some() 
                && path.extension().unwrap() == "html"
            })
            .collect();

        Ok(filtered_htmls)
    }

    pub async fn to_remote_html_container(&self) -> SyncResult<remote::html::Container> {
        let html_paths = self.html_paths().await?;
        let container = remote::html::Container::from_paths(html_paths).await?;

        Ok(container)
    }

    pub async fn to_fulltime_parser(&self, sc_type: RawType) -> SyncResult<fulltime::html::Parser> {
        let html_paths = self.html_paths().await?;
        let html_path = html_paths.get(0).unwrap();
        let parser = fulltime::html::Parser::from_path(
            html_path.clone(), 
            sc_type
        ).await;

        parser
    }
}