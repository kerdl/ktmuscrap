use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{
    sync::Arc,
    path::PathBuf
};

use crate::{
    SyncResult,
    data::{
        schedule::Page,
        json::{
            self,
            Path,
            Saving,
            Loading,
            DirectLoading
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
impl json::Saving<MiddleLast> for Last {}
impl Last {
    pub fn default(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            ft_weekly: Arc::new(RwLock::new(None)),
            ft_daily: Arc::new(RwLock::new(None)),
            r_weekly: Arc::new(RwLock::new(None)),
        };

        Arc::new(this)
    }

    fn from_middle(
        middle: Arc<MiddleLast>,
        path: PathBuf
    ) -> Arc<Self> {

        let this = Last {
            path,
            ft_weekly: Arc::new(RwLock::new(middle.ft_daily.clone())),
            ft_daily:  Arc::new(RwLock::new(middle.ft_weekly.clone())),
            r_weekly:  Arc::new(RwLock::new(middle.r_weekly.clone())),
        };

        Arc::new(this)
    }

    pub async fn load(path: PathBuf) -> SyncResult<Arc<Self>> {
        let middle = MiddleLast::load(path.clone()).await?;
        let primary = Self::from_middle(middle, path);

        Ok(primary)
    }

    pub async fn load_or_init(path: PathBuf) -> SyncResult<Arc<Self>> {
        let this;

        if path.exists() {
            this = Self::load(path).await?;
        } else {
            this = Self::default(path);
            this.save().await.unwrap();
        }

        Ok(this)
    }

    pub fn clone_cleared(self: Arc<Self>) -> Arc<Self> {
        Self::default(self.path.clone())
    }

    pub async fn is_cleared(self: Arc<Self>) -> bool {
        self.ft_daily.read().await.is_none()
        && self.ft_weekly.read().await.is_none()
        && self.r_weekly.read().await.is_none()
    }

    pub async fn clear_ft_daily(self: Arc<Self>) {
        *self.ft_daily.write().await = None;
        self.poll_save();
    }

    pub async fn ft_daily_is_none(self: Arc<Self>) -> bool {
        self.ft_daily.read().await.is_none()
    }

    pub async fn clear_ft_weekly(self: Arc<Self>) {
        *self.ft_weekly.write().await = None;
        self.poll_save();
    }

    pub async fn ft_weekly_is_none(self: Arc<Self>) -> bool {
        self.ft_weekly.read().await.is_none()
    }

    pub async fn clear_r_weekly(self: Arc<Self>) {
        *self.r_weekly.write().await = None;
        self.poll_save();
    }

    pub async fn r_weekly_is_none(self: Arc<Self>) -> bool {
        self.r_weekly.read().await.is_none()
    }
}


#[derive(Serialize, Deserialize)]
pub struct MiddleLast {
    #[serde(skip)]
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
impl json::DirectSaving for MiddleLast {}
impl json::DirectLoading for MiddleLast {}
