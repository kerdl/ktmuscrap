pub mod table;

use log::{info, warn};
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
    data::schedule::{
        remote::html::Container as HtmlContainer,
        Page
    }, 
    SyncResult,
    REMOTE_INDEX_PATH,
    REMOTE_INDEX, parse::fulltime,
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
    EnumString,
    Hash
)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    FtWeekly,
    FtDaily,
    RWeekly
}


#[derive(new, Debug)]
pub struct Zip {
    sc_type: Type,
    pub content: RwLock<Option<Bytes>>,
}
impl Zip {
    pub fn from_sc_type(sc_type: Type) -> Zip {
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

            for ignored_path in REMOTE_INDEX.get().unwrap().ignored.read().await.iter() {

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

#[derive(new, Clone, Debug)]
pub struct Schedule {
    pub zip: Arc<RwLock<Zip>>,
    pub parsed: Arc<RwLock<Option<Arc<Page>>>>
}
impl Schedule {
    pub fn from_sc_type(sc_type: Type) -> Schedule {
        Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(sc_type))),
            parsed: Arc::new(RwLock::new(None))
        }
    }

    pub async fn clear_parsed(&self) {
        *self.parsed.write().await = None;
    }
}

#[derive(new, Clone, Debug)]
pub struct Container {
    path: PathBuf,
    /// ## `F`ull`t`ime `weekly` schdule ZIP file
    pub ft_weekly: Arc<Schedule>,
    /// ## `F`ull`t`ime `daily` schedule ZIP file
    pub ft_daily: Arc<Schedule>,
    /// ## `R`emote `weekly` schedule ZIP file
    pub r_weekly: Arc<Schedule>,
}
impl Container {
    fn from_serde_container(serde_container: SerdeContainer) -> Container {
        let ft_weekly = Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(Type::FtWeekly))),
            parsed: Arc::new(RwLock::new(serde_container.ft_weekly))
        };

        let ft_daily = Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(Type::FtDaily))),
            parsed: Arc::new(RwLock::new(serde_container.ft_daily))
        };

        let r_weekly = Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(Type::RWeekly))),
            parsed: Arc::new(RwLock::new(serde_container.r_weekly))
        };

        Container::new(
            serde_container.path,
            Arc::new(ft_weekly),
            Arc::new(ft_daily),
            Arc::new(r_weekly)
        )
    }

    pub fn from_path(path: PathBuf) -> Container {
        Container {
            path,
            ft_weekly: Arc::new(Schedule::from_sc_type(Type::FtWeekly)),
            ft_daily: Arc::new(Schedule::from_sc_type(Type::FtDaily)),
            r_weekly: Arc::new(Schedule::from_sc_type(Type::RWeekly))
        }
    }

    pub async fn save(&self) {
        let serde_container = Arc::new(
            SerdeContainer::from_container(&self).await
        );

        tokio::spawn(async move {
            serde_container.save().await
        });
    }

    pub fn load(path: PathBuf) -> SyncResult<Container> {

        let serde_container = SerdeContainer::load(path)?;
        let container = Container::from_serde_container(serde_container);

        Ok(container)
    }

    pub async fn load_or_init(path: PathBuf) -> SyncResult<Container> {
        let container;

        if !path.exists() {
            container = Container::from_path(path);
            container.save().await;
        } else {
            container = Container::load(path)?;
        }

        Ok(container)
    }

    /// ## Remove all folders of schedules
    pub async fn remove_folders_if_exists(self: Arc<Self>) -> DynResult<()> {
        self.ft_weekly.zip.read().await.remove_folder_if_exists().await?;
        self.ft_daily.zip.read().await.remove_folder_if_exists().await?;
        self.r_weekly.zip.read().await.remove_folder_if_exists().await?;

        Ok(())
    }

    /// ## Clear all content loaded into RAM
    pub async fn clear_loaded(self: Arc<Self>) {
        self.ft_weekly.zip.read().await.clear_content().await;
        self.ft_daily.zip.read().await.clear_content().await;
        self.r_weekly.zip.read().await.clear_content().await;
    }

    /// ## Remove all folders and clear all content loaded into RAM
    pub async fn delete(self: Arc<Self>) {
        let _ = self.clone().remove_folders_if_exists().await;
        self.clone().clear_loaded().await;
    }
}

#[derive(new, Serialize, Deserialize)]
struct SerdeContainer {
    path: PathBuf,
    ft_weekly: Option<Arc<Page>>,
    ft_daily: Option<Arc<Page>>,
    r_weekly: Option<Arc<Page>>,
}
impl SerdeContainer {
    pub async fn from_container(container: &Container) -> SerdeContainer {
        SerdeContainer::new(
            container.path.clone(),
            container.ft_weekly.parsed.read().await.as_ref().map(|page| page.clone()),
            container.ft_daily.parsed.read().await.as_ref().map(|page| page.clone()),
            container.r_weekly.parsed.read().await.as_ref().map(|page| page.clone()),
        )
    }

    pub async fn save(self: Arc<Self>) -> SyncResult<()> {
    
        if let Err(err) = tokio::task::spawn_blocking(move || -> SyncResult<()> {
            let ser = serde_json::ser::to_string_pretty(&self)?;
            std::fs::write(&self.path, ser)?;

            Ok(())
        }).await? {
            warn!("error while saving data::schedule::raw::SerdeContainer {:?}", err);
        }

        Ok(())
    }

    pub fn load(path: PathBuf) -> SyncResult<SerdeContainer> {
        let de = std::fs::read_to_string(path)?;
        let last: SerdeContainer = serde_json::de::from_str(&de)?;

        Ok(last)
    }
}


#[derive(new, Debug)]
pub struct LatestIndex {
    pub sha256: String,
}

#[derive(new, Debug)]
pub struct Index {
    pub path: PathBuf,
    pub latest: LatestIndex,
    pub ignored: Arc<RwLock<HashSet<PathBuf>>>
}
impl Index {
    fn from_serde_index(serde_index: SerdeIndex) -> Index {
        Index::new(
            serde_index.path,
            LatestIndex::new("".to_owned()),
            Arc::new(RwLock::new(serde_index.ignored))
        )
    }

    pub fn from_path(path: PathBuf) -> Index {
        Index::new(
            path,
            LatestIndex::new("".to_owned()),
            Arc::new(RwLock::new(HashSet::new()))
        )
    }

    pub async fn save(&self) {
        let serde_index = Arc::new(
            SerdeIndex::from_index(&self).await
        );

        tokio::spawn(async move {
            serde_index.save().await
        });
    }

    pub async fn load(path: PathBuf) -> SyncResult<Index> {
        let serde_index = SerdeIndex::load(path).await?;

        let index = Index::from_serde_index(serde_index);

        Ok(index)
    }

    pub async fn load_or_init(path: PathBuf) -> SyncResult<Index> {
        let index;

        if !path.exists() {
            index = Index::from_path(path);
            index.save().await;
        } else {
            index = Index::load(path).await?;
        }

        Ok(index)
    }

    pub async fn remove_ignored(&self) {
        for path in self.ignored.read().await.iter() {
            let path = path.clone();

            tokio::spawn(async move {
                if path.exists() && path.is_file() {
                    tokio::fs::remove_file(path).await.unwrap();
                }
            });
        }
    }
}

#[derive(new, Serialize, Deserialize)]
struct SerdeIndex {
    path: PathBuf,
    pub ignored: HashSet<PathBuf>
}
impl SerdeIndex {
    pub async fn from_index(index: &Index) -> SerdeIndex {
        SerdeIndex::new(
            index.path.clone(), 
            index.ignored.read().await.clone()
        )
    }

    pub async fn save(self: Arc<Self>) -> SyncResult<()> {
        if let Err(err) = tokio::task::spawn_blocking(move || -> SyncResult<()> {

            let ser = serde_json::ser::to_string_pretty(&self)?;
            std::fs::write(&self.path, ser)?;

            Ok(())

        }).await? {
            warn!("error while saving data::schedule::raw::SerdeIndex {:?}", err);
        };

        Ok(())
    }

    pub async fn load(path: PathBuf) -> SyncResult<SerdeIndex> {
        let de = tokio::fs::read_to_string(path).await?;
        let index: SerdeIndex = tokio::task::spawn_blocking(
            move || -> serde_json::Result<SerdeIndex> {
                serde_json::de::from_str(&de)
            }
        ).await??;

        Ok(index)
    }
}