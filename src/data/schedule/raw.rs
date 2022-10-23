use actix_web::web::{Bytes, Buf};
use log::info;
use zip::read::ZipArchive;
use serde_derive::Serialize;
use tokio::sync::RwLock;
use std::{path::{Path, PathBuf}, io::Cursor, sync::Arc};

use crate::DynResult;
use super::error;


#[derive(Serialize, Debug, Clone)]
pub enum Type {
    FtWeekly,
    FtDaily,
    RWeekly
}
impl Type {
    pub fn from_string(&self, string: &str) -> Option<Type> {
        match string {
            "ft_weekly" => Some(Type::FtWeekly),
            "ft_daily"  => Some(Type::FtDaily),
            "r_weekly"  => Some(Type::RWeekly),
            _           => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Type::FtWeekly => "ft_weekly",
            Type::FtDaily  => "ft_daily",
            Type::RWeekly  => "r_weekly"
        }
    }
}


pub struct Html {
    sc_type: Type,
    content: RwLock<String>,
}
impl Html {
    pub fn new(sc_type: Type, content: RwLock<String>) -> Html {
        Html { sc_type, content }
    }

    pub fn from_zip(raw_zip: Zip) -> Html {
        unimplemented!()

        //let content = raw_zip.extract();

        //RawHtml::new(raw_zip.sc_type, content)
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

    pub async fn del_content(&self) {
        let mut field = self.content.write().await;
        field.take();
    }

    /// ## Generate potential path for this schedule type
    pub fn path(&self) -> PathBuf {
        // naming the directory the same as schedule type
        let dir_name = self.sc_type.to_str();
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
}
impl Default for Container {
    fn default() -> Container {
        let ft_weekly = Arc::new(RwLock::new(Zip::new(Type::FtWeekly, RwLock::new(None))));
        let ft_daily  = Arc::new(RwLock::new(Zip::new(Type::FtDaily, RwLock::new(None))));
        let r_weekly  = Arc::new(RwLock::new(Zip::new(Type::RWeekly, RwLock::new(None))));

        Container::new(ft_weekly, ft_daily, r_weekly)
    }
}
