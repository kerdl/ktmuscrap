use log::{debug, warn};
use chrono::{NaiveDateTime, Utc, Duration};
use ::zip::ZipArchive;
use async_trait::async_trait;
use async_recursion::async_recursion;
use serde_derive::{Serialize, Deserialize};
use reqwest;
use actix_web::web::Bytes;
use tokio::{sync::RwLock, task::JoinHandle};
use std::{
    io::Cursor,
    path::PathBuf,
    sync::Arc,
    collections::HashSet
};

use crate::{
    SyncResult,
    data::json::{self, SavingLoading},
    fs, parse
};
use super::{
    Type,
    ignored,
    fulltime,
    remote,
    error,
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

    pub async fn update_all(self: Arc<Self>) -> Result<(), error::UnpackError> {
        debug!("updating all schedules in Index");

        let mut handles = vec![];

        self.updated_now().await;

        for schedule in self.types.iter() {
            let schedule = schedule.clone();

            let handle = tokio::spawn(async move {
                let bytes = schedule.refetch_until_success().await;
                schedule.unpack(bytes).await?;

                Ok::<(), error::UnpackError>(())
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap()?;
        }

        self.clone().poll_save();

        debug!("fetched and unpacked all successfully");

        self.clone().post_update_all().await;

        Ok(())
    }

    async fn post_update_all(self: Arc<Self>) {
        /* 
        let ft_daily = self.types.iter().find(
            |schedule| schedule.sc_type == Type::FtDaily
        ).unwrap();
        let ft_weekly = self.types.iter().find(
            |schedule| schedule.sc_type == Type::FtWeekly
        ).unwrap();
        let r_weekly = self.types.iter().find(
            |schedule| schedule.sc_type == Type::RWeekly
        ).unwrap();
    

        parse::daily(
            ft_daily.dir(),
            r_weekly.dir(),
            last,
            raw_last
        ).await;

        parse::weekly(
            ft_weekly.dir(),
            r_weekly.dir(),
            last,
            raw_last
        ).await;
        */
    }

    /// # DO NOT AWAIT!!!
    pub fn update_forever(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let next = self.clone().next_update().await;
                let mut until = self.clone().until_next_update().await;
    
                if until < Duration::zero() {
                    until = Duration::zero()
                }

                debug!("next fetch at {} (in {} secs)", next, until.num_seconds());
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


#[derive(Debug, Clone)]
pub struct Schedule {
    root: PathBuf,

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Arc<RwLock<Option<PathBuf>>>,
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

    pub async fn get_latest(&self) -> SyncResult<Option<PathBuf>> {
        let dir = self.dir();

        let path = match self.sc_type {
            Type::FtDaily =>  fulltime::latest(&dir).await.unwrap(),
            Type::FtWeekly => fulltime::latest(&dir).await.unwrap(),
            Type::RWeekly =>  remote::latest(&dir).await.unwrap()
        };

        *self.latest.write().await = path.clone();

        Ok(path)
    }

    pub async fn get_ignored(&self) -> tokio::io::Result<Option<HashSet<PathBuf>>> {
        let latest = self.latest.read().await;
        if latest.is_none() {
            return Ok(None)
        }
        let latest = latest.as_ref().unwrap();

        let ignored = ignored::except(&latest).await?;

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
                _ => panic!("pls handle this for {}: {:?}", self.sc_type, error)
            }
        }
        
        fetch_result.unwrap()
    }

    pub async fn unpack(self: Arc<Self>, bytes: Bytes) -> Result<(), error::UnpackError> {
        let dir = self.dir();

        debug!("unpacking {:?}", dir);

        if dir.exists() {
            debug!("{:?} exists, removing before unpacking", dir);
            if let Err(error) = tokio::fs::remove_dir_all(&dir).await {
                return Err(error::UnpackError::Io(error));
            }
        }

        if !dir.exists() {
            debug!("{:?} doesn't exist, creating to unpack there", dir);
            if let Err(error) = tokio::fs::create_dir(&dir).await {
                return Err(error::UnpackError::Io(error));
            }
        }

        tokio::task::spawn_blocking(move || -> Result<(), error::UnpackError> {
            let cursor = Cursor::new(bytes);
            let archive_result = ZipArchive::new(cursor);

            if let Err(error) = archive_result {
                return Err(error::UnpackError::Zip(error));
            }

            let mut archive = archive_result.unwrap();

            if let Err(error) = archive.extract(dir) {
                return Err(error::UnpackError::Zip(error));
            }
    
            Ok(())
        }).await.unwrap()?;

        self.post_unpack().await;

        Ok(())
    }

    async fn post_unpack(self: Arc<Self>) {
        self.purge_ignored().await.unwrap();
        self.get_latest().await.unwrap();
        self.get_ignored().await.unwrap();

        tokio::spawn(async move {
            if let Err(error) = self.purge_ignored().await {
                warn!("error while purging ignored: {:?}", error);
            }
        });
    }

    pub async fn purge_ignored(&self) -> tokio::io::Result<()> {
        fs::remove::from_set(self.ignored.read().await.clone()).await
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct MiddleSchedule {
    root: PathBuf,

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Option<PathBuf>,
    pub ignored: HashSet<PathBuf>,
}
impl MiddleSchedule {
    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }
}