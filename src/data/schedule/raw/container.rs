use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{
    sync::Arc,
    path::PathBuf
};

use crate::{
    DynResult,
    data::{
        schedule::Page,
        json::{
            self,
            Path,
        }
    },
};
use super::{
    Zip,
    Type as RawType
};


#[derive(Clone, Debug)]
pub struct Schedule {
    pub zip: Arc<RwLock<Zip>>,
    pub parsed: Arc<RwLock<Option<Arc<Page>>>>
}
impl Schedule {
    pub fn from_sc_type(sc_type: RawType) -> Schedule {
        Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(sc_type))),
            parsed: Arc::new(RwLock::new(None))
        }
    }

    pub async fn clear_parsed(&self) {
        *self.parsed.write().await = None;
    }
}

#[derive(Clone, Debug)]
pub struct Container {
    path: PathBuf,
    /// ## `F`ull`t`ime `weekly` schdule ZIP file
    pub ft_weekly: Arc<Schedule>,
    /// ## `F`ull`t`ime `daily` schedule ZIP file
    pub ft_daily: Arc<Schedule>,
    /// ## `R`emote `weekly` schedule ZIP file
    pub r_weekly: Arc<Schedule>,
}
impl json::Path for Container {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DefaultFromPath for Container {
    fn default_from_path(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            ft_weekly: Arc::new(Schedule::from_sc_type(RawType::FtWeekly)),
            ft_daily: Arc::new(Schedule::from_sc_type(RawType::FtDaily)),
            r_weekly: Arc::new(Schedule::from_sc_type(RawType::RWeekly)),
        };

        Arc::new(this)
    }
}
impl json::FromMiddle<MiddleContainer> for Container {
    fn from_middle(middle: Arc<MiddleContainer>) -> Arc<Self> {
        let ft_weekly = Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(RawType::FtWeekly))),
            parsed: Arc::new(RwLock::new(middle.ft_weekly.clone()))
        };

        let ft_daily = Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(RawType::FtDaily))),
            parsed: Arc::new(RwLock::new(middle.ft_daily.clone()))
        };

        let r_weekly = Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(RawType::RWeekly))),
            parsed: Arc::new(RwLock::new(middle.r_weekly.clone()))
        };

        let this = Container {
            path: middle.path(),
            ft_weekly: Arc::new(ft_weekly),
            ft_daily: Arc::new(ft_daily),
            r_weekly: Arc::new(r_weekly)
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleContainer> for Container {
    async fn to_middle(&self) -> MiddleContainer {
        let middle = MiddleContainer {
            path: self.path(),
            ft_daily: self.ft_daily.parsed.read().await.clone(),
            ft_weekly: self.ft_weekly.parsed.read().await.clone(),
            r_weekly: self.r_weekly.parsed.read().await.clone()
        };

        middle
    }
}
impl json::SavingLoading<MiddleContainer> for Container {}
impl json::LoadOrInit<MiddleContainer> for Container {}
impl Container {
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

#[derive(Serialize, Deserialize)]
pub struct MiddleContainer {
    path: PathBuf,
    ft_weekly: Option<Arc<Page>>,
    ft_daily: Option<Arc<Page>>,
    r_weekly: Option<Arc<Page>>,
}
impl json::Path for MiddleContainer {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSavingLoading for MiddleContainer {}
