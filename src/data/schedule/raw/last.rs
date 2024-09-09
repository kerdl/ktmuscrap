use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf};

use crate::{
    data::{
        schedule::Page,
        json::{
            self,
            Path,
            Saving,
            DirectLoading,
        }
    },
    SyncResult
};


/// # Stores last converted schedules
#[derive(Clone, Debug)]
pub struct Last {
    path: PathBuf,
    pub groups: Arc<RwLock<Option<Arc<Page>>>>,
    pub teachers: Arc<RwLock<Option<Arc<Page>>>>,
}
impl json::Path for Last {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl json::ToMiddle<MiddleLast> for Last {
    async fn to_middle(&self) -> MiddleLast {
        MiddleLast {
            path: self.path(),
            groups: self.groups.read().await.as_ref().map(
                |page| page.clone()
            ),
            teachers: self.teachers.read().await.as_ref().map(
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
            groups: Arc::new(RwLock::new(None)),
            teachers: Arc::new(RwLock::new(None)),
        };

        Arc::new(this)
    }

    fn from_middle(middle: Arc<MiddleLast>, path: PathBuf) -> Arc<Self> {
        let this = Last {
            path,
            groups: Arc::new(RwLock::new(middle.groups.clone())),
            teachers: Arc::new(RwLock::new(middle.teachers.clone())),
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
        self.groups.read().await.is_none()
        && self.teachers.read().await.is_none()
    }

    pub async fn set_groups(self: Arc<Self>, page: Page) {
        *self.groups.write().await = {
            Some(Arc::new(page))
        };
        self.poll_save()
    }

    pub async fn clear_groups(self: Arc<Self>) {
        *self.groups.write().await = None;
        self.poll_save()
    }

    pub async fn set_teachers(self: Arc<Self>, page: Page) {
        *self.teachers.write().await = {
            Some(Arc::new(page))
        };
        self.poll_save()
    }

    pub async fn clear_teachers(self: Arc<Self>) {
        *self.teachers.write().await = None;
        self.poll_save()
    }
}

#[derive(Serialize, Deserialize)]
pub struct MiddleLast {
    #[serde(skip)]
    path: PathBuf,
    groups: Option<Arc<Page>>,
    teachers: Option<Arc<Page>>,
}
impl json::Path for MiddleLast {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSaving for MiddleLast {}
impl json::DirectLoading for MiddleLast {}
