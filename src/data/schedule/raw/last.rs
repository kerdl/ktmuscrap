use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{
    sync::Arc,
    path::PathBuf
};

use crate::{
    data::{
        schedule::Page,
        json::{
            self,
            Path,
            SavingLoading
        }
    },
};


#[derive(Clone, Debug)]
pub struct Last {
    path: PathBuf,
    /// ## `F`ull`t`ime `weekly` schedule `Page`
    pub ft_weekly: Arc<RwLock<Option<Arc<Page>>>>,
    /// ## `F`ull`t`ime `daily` schedule `Page`
    pub ft_daily: Arc<RwLock<Option<Arc<Page>>>>,
    /// ## `R`emote `weekly` schedule `Page`
    pub r_weekly: Arc<RwLock<Option<Arc<Page>>>>,
}
impl json::Path for Last {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DefaultFromPath for Last {
    fn default_from_path(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            ft_weekly: Arc::new(RwLock::new(None)),
            ft_daily: Arc::new(RwLock::new(None)),
            r_weekly: Arc::new(RwLock::new(None)),
        };

        Arc::new(this)
    }
}
impl json::FromMiddle<MiddleLast> for Last {
    fn from_middle(middle: Arc<MiddleLast>) -> Arc<Self> {
        let this = Last {
            path:      middle.path(),
            ft_weekly: Arc::new(RwLock::new(middle.ft_daily.clone())),
            ft_daily:  Arc::new(RwLock::new(middle.ft_weekly.clone())),
            r_weekly:  Arc::new(RwLock::new(middle.r_weekly.clone())),
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleLast> for Last {
    async fn to_middle(&self) -> MiddleLast {
        let middle = MiddleLast {
            path:      self.path(),
            ft_daily:  self.ft_daily.read().await.clone(),
            ft_weekly: self.ft_weekly.read().await.clone(),
            r_weekly:  self.r_weekly.read().await.clone()
        };

        middle
    }
}
impl json::SavingLoading<MiddleLast> for Last {}
impl json::LoadOrInit<MiddleLast> for Last {}
impl Last {
    pub async fn clear_ft_daily(self: Arc<Self>) {
        *self.ft_daily.write().await = None;
        self.poll_save();
    }

    pub async fn clear_ft_weekly(self: Arc<Self>) {
        *self.ft_weekly.write().await = None;
        self.poll_save();
    }

    pub async fn clear_r_weekly(self: Arc<Self>) {
        *self.r_weekly.write().await = None;
        self.poll_save();
    }
}


#[derive(Serialize, Deserialize)]
pub struct MiddleLast {
    path: PathBuf,
    ft_weekly: Option<Arc<Page>>,
    ft_daily: Option<Arc<Page>>,
    r_weekly: Option<Arc<Page>>,
}
impl json::Path for MiddleLast {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSavingLoading for MiddleLast {}
