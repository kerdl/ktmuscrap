mod settings;
mod schedule;

use actix_web::web::Bytes;
use chrono::Duration;
use log::{info, debug};
use tokio::sync::{RwLock, mpsc, watch};
use std::{path::PathBuf, sync::Arc, collections::HashSet};
use crate::{
    compare::{self, DetailedCmp},
    data::{
        json::Saving,
        schedule::{raw::{self, Last}, Notify},
        container::{
            settings::Settings,
            schedule::Schedule
        }
    },
    merge, parse, string, SyncResult
};


#[derive(Debug)]
pub struct Container {
    pub dir: PathBuf,

    pub settings: Arc<Settings>,
    pub schedule: Arc<Schedule>,
}
impl Container {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Container> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let this = Container {
            dir: dir.clone(),
            settings: Settings::load_or_init(
                dir.join("settings.json")
            ).await?,
            schedule: Schedule::default_from_dir(
                dir.join("schedule")
            ).await?
        };

        Ok(this)
    }
}