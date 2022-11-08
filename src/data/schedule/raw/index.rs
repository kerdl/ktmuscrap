use async_recursion::async_recursion;
use chrono::{NaiveDateTime, NaiveTime, Utc, Duration};
use log::{debug, warn};
use ::zip::ZipArchive;
use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use reqwest;
use actix_web::web::Bytes;
use tokio::{sync::RwLock, task::JoinHandle};
use zip::result::ZipResult;
use std::{io::Cursor, path::PathBuf, sync::Arc, collections::HashSet, any::Any};

use crate::{
    SyncResult,
    data::json::{self, SavingLoading},
    fs
};
use super::{
    Type,
    fulltime,
    remote, ignored
};


#[derive(Debug)]
pub struct Index {
    path: PathBuf,

    pub updated: Arc<RwLock<NaiveDateTime>>,
    pub types: Vec<Arc<Schedule>>
}
impl json::Path for Index {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DefaultFromPath for Index {
    fn default_from_path(path: PathBuf) -> Arc<Self> {
        let dir = path.parent().unwrap_or(&path).to_path_buf();

        let this = Self {
            path: path.clone(),
            updated: Arc::new(RwLock::new(
                NaiveDateTime::from_timestamp(0, 0)
            )),
            types: vec![
                Schedule::default_ft_daily(dir.clone()),
                Schedule::default_ft_weekly(dir.clone()),
                Schedule::default_r_weekly(dir)
            ]
        };

        Arc::new(this)
    }
}
impl json::FromMiddle<MiddleIndex> for Index {
    fn from_middle(middle: Arc<MiddleIndex>) -> Arc<Self> {
        let types = {
            let mut types = vec![];
            for sc in middle.types.iter() {
                let primary = Schedule::from_middle(Arc::new(sc.clone()));
                types.push(primary);
            }
            types
        };

        let this = Self {
            path: middle.path.clone(),
            updated: Arc::new(RwLock::new(middle.updated.clone())),
            types
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleIndex> for Index {
    async fn to_middle(&self) -> MiddleIndex {
        let types = {
            let mut types = vec![];
            for sc in self.types.iter() {
                types.push(Schedule::to_middle(sc).await)
            }
            types
        };

        MiddleIndex {
            path: self.path.clone(),
            updated: self.updated.read().await.clone(),
            types
        }
    }
}
impl json::SavingLoading<MiddleIndex> for Index {}
impl json::LoadOrInit<MiddleIndex> for Index {}
impl Index {
    pub fn update_period(&self) -> Duration {
        Duration::minutes(10)
    }

    pub async fn next_update(&self) -> NaiveDateTime {
        *self.updated.read().await + self.update_period()
    }

    pub async fn until_next_update(&self) -> Duration {
        self.next_update().await - Utc::now().naive_utc()
    }

    pub async fn needs_update(&self) -> bool {
        self.until_next_update().await < Duration::zero()
    }

    pub async fn updated_now(&self) {
        *self.updated.write().await = Utc::now().naive_utc()
    }

    pub async fn update_all(self: Arc<Self>) -> SyncResult<()> {
        debug!("updating all schedules in Index");

        let mut handles = vec![];

        self.updated_now().await;

        for schedule in self.types.iter() {
            let schedule = schedule.clone();

            let handle = tokio::spawn(async move {
                let bytes = schedule.refetch_until_success().await;
                schedule.unpack(bytes).await?;

                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await??;
        }

        self.poll_save();

        debug!("fetched and unpacked all successfully");

        Ok(())
    }

    /// # DO NOT AWAIT!!!
    pub fn update_forever(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let mut until = self.clone().until_next_update().await;
    
                if until < Duration::zero() {
                    until = Duration::zero()
                }
    
                tokio::time::sleep(until.to_std().unwrap()).await;
    
                self.clone().update_all().await.unwrap();
            }
        })
    }
}


#[derive(Serialize, Deserialize)]
pub struct MiddleIndex {
    path: PathBuf,

    pub updated: NaiveDateTime,
    pub types: Vec<MiddleSchedule>
}
impl json::Path for MiddleIndex {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSavingLoading for MiddleIndex {}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Latest {
    pub path: PathBuf,
    pub sha256: String,
}
impl Latest {
    pub async fn from_dir(
        dir: PathBuf,
        sc_type: &Type
    ) -> SyncResult<Option<Arc<Latest>>> {

        let path = match sc_type {
            Type::FtDaily =>  fulltime::latest(&dir).await?,
            Type::FtWeekly => fulltime::latest(&dir).await?,
            Type::RWeekly =>  remote::latest(&dir).await?
        };
        if path.is_none() {
            return Ok(None)
        }
        let path = path.unwrap();

        let sha256 = fs::hash::get_sha256(&path).await?;

        let this = Self {
            path,
            sha256
        };

        Ok(Some(Arc::new(this)))
    }
}


#[derive(Debug, Clone)]
pub struct Schedule {
    root: PathBuf,

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Arc<RwLock<Option<Arc<Latest>>>>,
    pub ignored: Arc<RwLock<HashSet<PathBuf>>>,
}
impl json::FromMiddle<MiddleSchedule> for Schedule {
    fn from_middle(middle: Arc<MiddleSchedule>) -> Arc<Self> {
        let this = Schedule {
            root:         middle.root.clone(),
            sc_type:      middle.sc_type.clone(),
            url:          middle.url.clone(),
            friendly_url: middle.friendly_url.clone(),
            latest:       Arc::new(RwLock::new(middle.latest.clone())),
            ignored:      Arc::new(RwLock::new(middle.ignored.clone())),
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleSchedule> for Schedule {
    async fn to_middle(&self) -> MiddleSchedule {
        let mid = MiddleSchedule {
            root:         self.root.clone(),
            sc_type:      self.sc_type.clone(),
            url:          self.url.clone(),
            friendly_url: self.friendly_url.clone(),
            latest:       self.latest.read().await.clone(),
            ignored:      self.ignored.read().await.clone()
        };

        mid
    }
}
impl Schedule {
    pub fn default_ft_daily(root: PathBuf) -> Arc<Schedule> {
        let this = Schedule {
            root,
            sc_type:      Type::FtDaily,
            url:          "https://docs.google.com/document/d/1gsE6aikIQ1umKSQWVnyn3_59mnGQQU8O/export?format=zip".to_owned(),
            friendly_url: "https://docs.google.com/document/d/1gsE6aikIQ1umKSQWVnyn3_59mnGQQU8O".to_owned(),
            latest:       Arc::new(RwLock::new(None)),
            ignored:      Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_ft_weekly(root: PathBuf) -> Arc<Schedule> {
        let this = Schedule {
            root,
            sc_type:      Type::FtWeekly,
            url:          "https://docs.google.com/document/d/1FH4ctIgRX1fWjIPoboXWieEYVMDYSlg4/export?format=zip".to_owned(),
            friendly_url: "https://docs.google.com/document/d/1FH4ctIgRX1fWjIPoboXWieEYVMDYSlg4".to_owned(),
            latest:       Arc::new(RwLock::new(None)),
            ignored:      Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_r_weekly(root: PathBuf) -> Arc<Schedule> {
        let this = Schedule {
            root,
            sc_type:      Type::RWeekly,
            url:          "https://docs.google.com/spreadsheets/d/1SWv7ARLLC6S_FjIzzhUz0kzGCdG53t9xL68VPoiYlnA/export?format=zip".to_owned(),
            friendly_url: "https://docs.google.com/spreadsheets/d/1SWv7ARLLC6S_FjIzzhUz0kzGCdG53t9xL68VPoiYlnA".to_owned(),
            latest:       Arc::new(RwLock::new(None)),
            ignored:      Arc::new(RwLock::new(HashSet::new())),
        };
        
        Arc::new(this)
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }

    pub fn refetch_period(&self) -> Duration {
        Duration::minutes(1)
    }

    pub async fn refresh_latest(&self) -> SyncResult<()> {
        let latest = Latest::from_dir(self.dir(), &self.sc_type).await?;
        *self.latest.write().await = latest;

        Ok(())
    }

    pub async fn get_ignored(&self) -> tokio::io::Result<Option<HashSet<PathBuf>>> {
        let latest = self.latest.read().await;
        if latest.is_none() {
            return Ok(None)
        }
        let latest = latest.as_ref().unwrap();

        let ignored = ignored::except(&latest.path.clone()).await?;

        let ignored_htmls = ignored.into_iter().filter(
            |path| if let Some(ext) = path.extension() {
                ext == "html"
            } else {
                false
            }
        ).collect::<HashSet<PathBuf>>();

        *self.ignored.write().await = ignored_htmls.clone();

        Ok(Some(ignored_htmls))
    }

    pub async fn fetch(&self) -> Result<Bytes, reqwest::Error> {
        let resp = reqwest::get(&self.url).await?;

        resp.bytes().await
    }

    pub async fn fetch_after(&self, after: Duration) -> Result<Bytes, reqwest::Error> {
        tokio::time::sleep(after.to_std().unwrap()).await;
        self.fetch().await
    }

    #[async_recursion]
    pub async fn refetch_until_success(&self) -> Bytes {
        debug!("fetching {}", self.sc_type);
        let fetch_result = self.fetch().await;

        if let Err(error) = &fetch_result {
            match error {
                error if error.is_status() => {
                    warn!(
                        "refetching {} because of status code {}",
                        self.sc_type,
                        error.status().unwrap()
                    );
    
                    self.fetch_after(self.refetch_period()).await.unwrap();
                }
                _ => panic!("pls handle this: {:?}", error)
            }
        }
        
        fetch_result.unwrap()
    }

    pub async fn unpack(&self, bytes: Bytes) -> SyncResult<()> {
        if self.dir().exists() {
            tokio::fs::remove_dir_all(self.dir()).await?;
        }

        let path = self.dir();

        tokio::task::spawn_blocking(move || -> ZipResult<Box<dyn Any + Send + Sync>> {
            let cursor = Cursor::new(bytes);
            let mut archive = ZipArchive::new(cursor)?;
            archive.extract(path)?;
    
            Ok(Box::new(()))
        }).await??;

        self.refresh_latest().await?;
        self.get_ignored().await?;
        for handle in self.purge_ignored().await {
            tokio::spawn(async move {
                if let Err(error) = handle.await {
                    warn!("error while purging ignored: {:?}", error)
                }
            });
        }

        Ok(())
    }

    pub async fn purge_ignored(&self) -> Vec<JoinHandle<tokio::io::Result<()>>> {
        fs::remove::from_set(self.ignored.read().await.clone())
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct MiddleSchedule {
    root: PathBuf,

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Option<Arc<Latest>>,
    pub ignored: HashSet<PathBuf>,
}
impl MiddleSchedule {
    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }
}