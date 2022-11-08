use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf};

use crate::{SyncResult, data::json::{self, Path, SavingLoading}};
use super::{raw, Page};


/// # Stores last converted schedule
#[derive(Clone, Debug)]
pub struct Last {
    path: PathBuf,

    pub weekly: Arc<RwLock<Option<Arc<Page>>>>,
    pub daily: Arc<RwLock<Option<Arc<Page>>>>
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
            weekly: Arc::new(RwLock::new(None)),
            daily: Arc::new(RwLock::new(None))
        };

        Arc::new(this)
    }
}
impl json::FromMiddle<MiddleLast> for Last {
    fn from_middle(middle: Arc<MiddleLast>) -> Arc<Self> {
        let this = Last {
            path: middle.path.clone(),
            weekly: Arc::new(RwLock::new(middle.weekly.clone())),
            daily: Arc::new(RwLock::new(middle.daily.clone())),
        };

        Arc::new(this)
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
        }
    }
}
impl json::SavingLoading<MiddleLast> for Last {}
impl json::LoadOrInit<MiddleLast> for Last {}
impl Last {
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

    pub async fn clear_from_raw_type(
        self: Arc<Self>,
        sc_type: &raw::Type
    ) {
        match sc_type {
            raw::Type::FtDaily => {
                self.clear_daily().await;
            }
            raw::Type::FtWeekly => {
                self.clear_weekly().await;
            }
            raw::Type::RWeekly => {
                self.clone().clear_daily().await;
                self.clear_weekly().await;
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MiddleLast {
    path: PathBuf,

    weekly: Option<Arc<Page>>,
    daily: Option<Arc<Page>>
}
impl json::Path for MiddleLast {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSavingLoading for MiddleLast {}