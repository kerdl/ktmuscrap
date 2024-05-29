use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf};

use crate::{data::json::{
    self,
    Path,
    Saving,
    Loading,
    DirectSaving,
    DirectLoading,
}, SyncResult};
use super::{raw, Page, TchrPage};


/// # Stores last converted schedule
#[derive(Clone, Debug)]
pub struct Last {
    path: PathBuf,

    pub weekly: Arc<RwLock<Option<Arc<Page>>>>,
    pub daily: Arc<RwLock<Option<Arc<Page>>>>,
    pub tchr_weekly: Arc<RwLock<Option<Arc<TchrPage>>>>,
    pub tchr_daily: Arc<RwLock<Option<Arc<TchrPage>>>>
}
impl json::Path for Last {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
#[async_trait]
impl json::ToMiddle<MiddleLast> for Last {
    async fn to_middle(&self) -> MiddleLast {
        MiddleLast {
            path: self.path(),
            weekly: self.weekly.read().await.as_ref().map(
                |page| page.clone()
            ),
            daily: self.daily.read().await.as_ref().map(
                |page| page.clone()
            ),
            tchr_weekly: self.tchr_weekly.read().await.as_ref().map(
                |page| page.clone()
            ),
            tchr_daily: self.tchr_daily.read().await.as_ref().map(
                |page| page.clone()
            ),
        }
    }
}
impl json::Saving<MiddleLast> for Last {}
impl Last {
    pub fn default(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            weekly: Arc::new(RwLock::new(None)),
            daily: Arc::new(RwLock::new(None)),
            tchr_daily: Arc::new(RwLock::new(None)),
            tchr_weekly: Arc::new(RwLock::new(None)),
        };

        Arc::new(this)
    }

    fn from_middle(middle: Arc<MiddleLast>, path: PathBuf) -> Arc<Self> {
        let this = Last {
            path,
            weekly: Arc::new(RwLock::new(middle.weekly.clone())),
            daily: Arc::new(RwLock::new(middle.daily.clone())),
            tchr_weekly: Arc::new(RwLock::new(middle.tchr_weekly.clone())),
            tchr_daily: Arc::new(RwLock::new(middle.tchr_daily.clone())),
        };

        Arc::new(this)
    }

    async fn load(path: PathBuf) -> SyncResult<Arc<Self>> {
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
            this.save().await?;
        }

        Ok(this)
    }

    pub fn clone_cleared(self: Arc<Self>) -> Arc<Self> {
        Self::default(self.path.clone())
    }

    pub async fn is_cleared(self: Arc<Self>) -> bool {
        self.weekly.read().await.is_none()
        && self.daily.read().await.is_none()
    }

    // groups

    pub async fn set_weekly(self: Arc<Self>, page: Page) {
        *self.weekly.write().await = {
            Some(Arc::new(page))
        };
        self.poll_save()
    }

    pub async fn clear_weekly(self: Arc<Self>) {
        *self.weekly.write().await = None;
        self.poll_save()
    }

    pub async fn set_daily(self: Arc<Self>, page: Page) {
        *self.daily.write().await = {
            Some(Arc::new(page))
        };
        self.poll_save()
    }

    pub async fn clear_daily(self: Arc<Self>) {
        *self.daily.write().await = None;
        self.poll_save()
    }

    // teachers

    pub async fn set_tchr_weekly(self: Arc<Self>, page: TchrPage) {
        *self.tchr_weekly.write().await = {
            Some(Arc::new(page))
        };
        self.poll_save()
    }

    pub async fn clear_tchr_weekly(self: Arc<Self>) {
        *self.tchr_weekly.write().await = None;
        self.poll_save()
    }

    pub async fn set_tchr_daily(self: Arc<Self>, page: TchrPage) {
        *self.tchr_daily.write().await = {
            Some(Arc::new(page))
        };
        self.poll_save()
    }

    pub async fn clear_tchr_daily(self: Arc<Self>) {
        *self.tchr_daily.write().await = None;
        self.poll_save()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MiddleLast {
    #[serde(skip)]
    path: PathBuf,

    weekly: Option<Arc<Page>>,
    daily: Option<Arc<Page>>,
    tchr_weekly: Option<Arc<TchrPage>>,
    tchr_daily: Option<Arc<TchrPage>>
}
impl json::Path for MiddleLast {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSaving for MiddleLast {}
impl json::DirectLoading for MiddleLast {}