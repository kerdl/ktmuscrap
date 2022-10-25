use log::info;
use chrono::NaiveDate;
use zip::read::ZipArchive;
use serde_derive::Serialize;
use strum_macros::{EnumString, Display};
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{path::PathBuf, io::Cursor, sync::Arc, collections::HashMap};

use crate::{DynResult, fs, api, parse};
use super::error;


#[derive(Serialize, Debug, Display, Clone, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    FtWeekly,
    FtDaily,
    RWeekly
}


pub struct Html {
    sc_type: Type,
    content: RwLock<String>,
}
impl Html {
    pub fn new(sc_type: Type, content: RwLock<String>) -> Html {
        Html { sc_type, content }
    }
}

pub struct Zip {
    sc_type: Type,
    pub content: RwLock<Option<Arc<Bytes>>>,
}
impl Zip {
    pub fn new(sc_type: Type, content: RwLock<Option<Arc<Bytes>>>) -> Zip {
        Zip { sc_type, content }
    }

    pub async fn set_content(&self, content: Bytes) {
        let mut field = self.content.write().await;
        *field = Some(Arc::new(content));
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
        let content_lock = self.content.read().await;

        if content_lock.is_none() {
            return Err(error::ExtractingEmptyContent.into())
        }

        let content = content_lock.as_ref().unwrap().clone();
        let cursor = Cursor::new(&content[..]);

        let dir_path = self.path();

        // if directory for this schedule type doesn't exist
        if !dir_path.exists() {
            // create it
            tokio::fs::create_dir(&dir_path).await?;
        }

        // parse archive
        let mut archive = ZipArchive::new(cursor)?;

        // extract it to directory we just created
        archive.extract(&dir_path)?;

        Ok(())
    }

    /// ## Latest schedule in this archive
    /// - only looks for `.html` files inside
    pub async fn latest_schedule(&self) -> DynResult<PathBuf> {
        // { <path to schedule>: <declared date in that schedule>}
        let mut path_date_map: HashMap<PathBuf, NaiveDate> = HashMap::new();

        let all_file_paths = fs::collect_file_paths(self.path()).await?;

        // vec of html files
         let html_paths = all_file_paths
            .into_iter()
            .filter(|path| {
                // has extension
                path.extension().is_some()
                // and that extension is "html"
                && path.extension().unwrap() == "html"
            })
            // create new vec of filtered files
            .collect::<Vec<PathBuf>>();

        if html_paths.len() < 1 {
            return Err(
                api::error::NoHtmls::new(
                    self.sc_type.clone()
                ).into()
            )
        }

        match self.sc_type {
            Type::FtWeekly | Type::FtDaily => {
                if html_paths.len() > 1 {
                    return Err(
                        api::error::MultipleHtmls::new(
                            self.sc_type.clone(), 
                            html_paths
                        ).into()
                    )
                }

                return Ok(html_paths.get(0).unwrap().to_owned())
            }
            Type::RWeekly => {
                for path in html_paths.into_iter() {
                    let parser = parse::remote::Html::from_path(&path).await?;
                    let date = parser.base_date();

                    if date.is_none() {
                        continue
                    }

                    let date = date.unwrap();

                    path_date_map.insert(path, date);
                }

                todo!()
            }
        }
    }
}

pub struct Container {
    /// ## `F`ull`t`ime `weekly` schdule ZIP file
    pub ft_weekly: Arc<RwLock<Zip>>,
    /// ## `F`ull`t`ime `daily` schedule ZIP file
    pub ft_daily: Arc<RwLock<Zip>>,
    /// ## `R`emote `weekly` schedule ZIP file
    pub r_weekly: Arc<RwLock<Zip>>,
}
impl Container {
    pub fn new(
        ft_weekly: Arc<RwLock<Zip>>, 
        ft_daily: Arc<RwLock<Zip>>, 
        r_weekly: Arc<RwLock<Zip>>
    ) -> Container {
        Container { ft_weekly, ft_daily, r_weekly }
    }

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
