//! # Indexing schedule files
//! 
//! Describing:
//! - a path to this index file
//! - what files to ignore inside the folder
//! - the latest's file hash

use derive_new::new;
use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{
    path::PathBuf,
    sync::Arc,
    collections::HashSet
};

use crate::{
    data::json::{
        self,
        Path
    }
};

#[derive(new, Debug)]
pub struct Ignored {
    path: PathBuf,
    pub ignored: Arc<RwLock<HashSet<PathBuf>>>
}
impl json::Path for Ignored {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DefaultFromPath for Ignored {
    fn default_from_path(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            ignored: Arc::new(RwLock::new(HashSet::new()))
        };

        Arc::new(this)
    }
}
impl json::FromMiddle<MiddleIgnored> for Ignored {
    fn from_middle(middle: Arc<MiddleIgnored>) -> Arc<Self> {
        let this = Self {
            path:    middle.path(),
            ignored: Arc::new(RwLock::new(middle.ignored.clone()))
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleIgnored> for Ignored {
    async fn to_middle(&self) -> MiddleIgnored {
        let mid = MiddleIgnored {
            path:    self.path(),
            ignored: self.ignored.read().await.clone()
        };

        mid
    }
}
impl json::SavingLoading<MiddleIgnored> for Ignored {}
impl json::LoadOrInit<MiddleIgnored> for Ignored {}
impl Ignored {
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
pub struct MiddleIgnored {
    #[serde(skip)]
    path: PathBuf,
    pub ignored: HashSet<PathBuf>
}
impl json::Path for MiddleIgnored {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSavingLoading for MiddleIgnored {}
