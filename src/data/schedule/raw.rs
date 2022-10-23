use actix_web::web::{Bytes, Buf};
use log::info;
use zip::read::ZipArchive;
use tokio::sync::RwLock;
use std::{path::{Path, PathBuf}, io::Cursor, sync::Arc};

use crate::DynResult;
use super::error;


#[derive(Debug, Clone)]
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
    content: Option<RwLock<Arc<Bytes>>>,
}
impl Zip {
    pub fn new(sc_type: Type, content: Option<RwLock<Arc<Bytes>>>) -> Zip {
        Zip { sc_type, content }
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
        if self.content.is_none() {
            return Err(error::ExtractingEmptyContent.into())
        }

        let content = self.content.as_ref().unwrap().read().await.clone();
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

    pub async fn set_ft_weekly(self: Arc<Self>, content: Bytes) {
        let mut field = self.ft_weekly.write().await;
        *field = Zip::new(Type::FtWeekly, Some(RwLock::new(Arc::new(content))))
    }

    pub async fn set_ft_daily(self: Arc<Self>, content: Bytes) {
        let mut field = self.ft_daily.write().await;
        *field = Zip::new(Type::FtDaily, Some(RwLock::new(Arc::new(content))))
    }

    pub async fn set_r_weekly(self: Arc<Self>, content: Bytes) {
        let mut field = self.r_weekly.write().await;
        *field = Zip::new(Type::RWeekly, Some(RwLock::new(Arc::new(content))))
    }
}
impl Default for Container {
    fn default() -> Container {
        let ft_weekly = Arc::new(RwLock::new(Zip::new(Type::FtWeekly, None)));
        let ft_daily  = Arc::new(RwLock::new(Zip::new(Type::FtDaily, None)));
        let r_weekly  = Arc::new(RwLock::new(Zip::new(Type::RWeekly, None)));

        Container::new(ft_weekly, ft_daily, r_weekly)
    }
}
