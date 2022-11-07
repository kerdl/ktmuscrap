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
    SyncResult,
    data::json::{
        self,
        SavingLoading,
        Path, DefaultFromPath
    }
};


#[derive(new, Debug)]
pub struct LatestIndex {
    pub sha256: String,
}

#[derive(new, Debug)]
pub struct Index {
    pub path: PathBuf,
    pub latest: LatestIndex,
    pub ignored: Arc<RwLock<HashSet<PathBuf>>>
}
impl json::Path for Index {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DefaultFromPath for Index {
    fn default_from_path(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            latest: LatestIndex::new("".to_owned()),
            ignored: Arc::new(RwLock::new(HashSet::new()))
        };

        Arc::new(this)
    }
}
impl json::FromMiddle<MiddleIndex> for Index {
    fn from_middle(middle: Arc<MiddleIndex>) -> Arc<Self> {
        let this = Self {
            path:    middle.path(),
            latest:  LatestIndex { sha256: "".to_owned() },
            ignored: Arc::new(RwLock::new(middle.ignored.clone()))
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleIndex> for Index {
    async fn to_middle(&self) -> MiddleIndex {
        let mid = MiddleIndex {
            path:    self.path(),
            ignored: self.ignored.read().await.clone()
        };

        mid
    }
}
impl json::SavingLoading<MiddleIndex> for Index {}
impl json::LoadOrInit<MiddleIndex> for Index {}
impl Index {
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
pub struct MiddleIndex {
    path: PathBuf,
    pub ignored: HashSet<PathBuf>
}
impl json::Path for MiddleIndex {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSavingLoading for MiddleIndex {}
