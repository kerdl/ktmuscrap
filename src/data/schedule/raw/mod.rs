pub mod table;

use log::info;
use derive_new::new;
use serde_derive::{Serialize, Deserialize};
use serde_json;
use zip::read::ZipArchive;
use strum_macros::{EnumString, Display};
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{path::PathBuf, io::Cursor, sync::Arc, collections::HashSet};

use crate::{
    DynResult, 
    fs, 
    data::schedule::remote::html::Container as HtmlContainer, 
    SyncResult,
    perf,
    REMOTE_SCHEDULE_INDEX_PATH,
    REMOTE_SCHEDULE_INDEX, parse::fulltime,
};
use super::error;


#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Display, 
    Clone, 
    PartialEq, 
    Eq, 
    EnumString
)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    FtWeekly,
    FtDaily,
    RWeekly
}


#[derive(new)]
pub struct Zip {
    sc_type: Type,
    pub content: RwLock<Option<Bytes>>,
}
impl Zip {
    pub async fn set_content(&self, content: Bytes) {
        let mut field = self.content.write().await;
        *field = Some(content);
    }

    pub async fn clear_content(&self) {
        let mut field = self.content.write().await;
        field.take();
    }

    /// ## Generate path for this schedule type
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

        if all_file_paths.contains(&REMOTE_SCHEDULE_INDEX_PATH) {

            for ignored_path in REMOTE_SCHEDULE_INDEX.read().await.ignored.iter() {

                if let Some(index) = {
                    all_file_paths.iter().position(|path| path == ignored_path)
                } {
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

    pub async fn to_remote_html_container(&self) -> SyncResult<HtmlContainer> {
        let html_paths = self.html_paths().await?;
        let container = HtmlContainer::from_paths(html_paths).await?;

        Ok(container)
    }

    pub async fn to_fulltime_parser(&self, sc_type: Type) -> SyncResult<fulltime::html::Parser> {
        let html_paths = self.html_paths().await?;
        let html_path = html_paths.get(0).unwrap();
        let parser = fulltime::html::Parser::from_path(
            html_path.clone(), 
            sc_type
        ).await;

        parser
    }
}

#[derive(new)]
pub struct Container {
    /// ## `F`ull`t`ime `weekly` schdule ZIP file
    pub ft_weekly: Arc<RwLock<Zip>>,
    /// ## `F`ull`t`ime `daily` schedule ZIP file
    pub ft_daily: Arc<RwLock<Zip>>,
    /// ## `R`emote `weekly` schedule ZIP file
    pub r_weekly: Arc<RwLock<Zip>>,
}
impl Container {
    /// ## Remove all folders of schedules
    pub async fn remove_folders_if_exists(self: Arc<Self>) -> DynResult<()> {
        self.ft_weekly.read().await.remove_folder_if_exists().await?;
        self.ft_daily.read().await.remove_folder_if_exists().await?;
        self.r_weekly.read().await.remove_folder_if_exists().await?;

        Ok(())
    }

    /// ## Clear all content loaded into RAM
    pub async fn clear_loaded(self: Arc<Self>) {
        self.ft_weekly.read().await.clear_content().await;
        self.ft_daily.read().await.clear_content().await;
        self.r_weekly.read().await.clear_content().await;
    }

    /// ## Remove all folders and clear all content loaded into RAM
    pub async fn delete(self: Arc<Self>) {
        let _ = self.clone().remove_folders_if_exists().await;
        self.clone().clear_loaded().await;
    }
}
impl Default for Container {
    fn default() -> Container {
        let ft_weekly = Arc::new(RwLock::new(Zip::new(Type::FtWeekly, RwLock::new(None))));
        let ft_daily  = Arc::new(RwLock::new(Zip::new(Type::FtDaily, RwLock::new(None))));
        let r_weekly  = Arc::new(RwLock::new(Zip::new(Type::RWeekly, RwLock::new(None))));

        Container::new(ft_weekly, ft_daily, r_weekly)
    }
}


#[derive(new, Serialize, Deserialize)]
pub struct Index {
    pub path: PathBuf,
    pub ignored: HashSet<PathBuf>
}
impl Index {
    pub fn load(path: PathBuf) -> SyncResult<Index> {
        let de = std::fs::read_to_string(path)?;
        let index: Index = serde_json::de::from_str(&de)?;

        Ok(index)
    }

    pub fn save(&self) -> SyncResult<()> {
        let ser = serde_json::ser::to_string_pretty(&self)?;
        std::fs::write(&self.path, ser)?;

        Ok(())
    }

    pub fn load_or_init(path: PathBuf) -> SyncResult<Index> {
        let index;

        if !path.exists() {
            index = Index::new(path.clone(), HashSet::new());
            index.save()?;
        } else {
            index = Index::load(path)?;
        }

        Ok(index)
    }

    pub async fn remove_ignored(&self) {
        for path in self.ignored.iter() {
            let path = path.clone();

            tokio::spawn(async move {
                if path.exists() && path.is_file() {
                    tokio::fs::remove_file(path).await.unwrap();
                }
            });
        }
    }
}