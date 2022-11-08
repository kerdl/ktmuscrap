use std::{path::PathBuf, sync::Arc};

use tokio::sync::RwLock;

use crate::SyncResult;

use super::{schedule::{Last, raw}, json::{DefaultFromPath, LoadOrInit}};


#[derive(Debug)]
pub struct Schedule {
    pub dir: PathBuf,

    pub last: Arc<Last>,
    pub raw_last: Arc<raw::Last>,
    pub index: Arc<raw::Index>
}
impl Schedule {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Schedule> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let this = Self {
            dir: dir.clone(),
            last: Last::load_or_init(
                dir.join("last.json")
            ).await?,
            raw_last: raw::Last::load_or_init(
                dir.join("raw_last.json")
            ).await?,
            index: raw::Index::load_or_init(
                dir.join("index.json")
            ).await?,
        };

        Ok(this)
    }
}

#[derive(Debug)]
pub struct Container {
    pub dir: PathBuf,

    pub schedule: Schedule,
}
impl Container {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Container> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let this = Container {
            dir: dir.clone(),
            schedule: Schedule::default_from_dir(
                dir.join("schedule")
            ).await?
        };

        Ok(this)
    }
}