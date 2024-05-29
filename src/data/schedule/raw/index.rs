use log::{debug, warn};
use chrono::{NaiveDateTime, Utc, Duration};
use ::zip::ZipArchive;
use async_trait::async_trait;
use async_recursion::async_recursion;
use serde_derive::{Serialize, Deserialize};
use reqwest;
use actix_web::web::Bytes;
use tokio::{
    sync::{
        RwLock,
        Mutex,
        mpsc
    },
    task::JoinHandle
};
use std::{
    io::Cursor,
    path::PathBuf,
    sync::Arc,
    collections::HashSet
};

use crate::{
    SyncResult,
    data::{json::{
        self,
        ToMiddle,
        Saving,
        DirectSaving,
        DirectLoading
    }, schedule::Interactor},
    fs
};
use super::{
    super::update,
    Type,
    ignored,
    fulltime,
    remote,
    error,
};

enum UpdateFinishType {
    Complete,
    LockRelease
}

#[derive(Debug)]
pub struct Index {
    fetch: bool,
    path: PathBuf,
    updated_tx: mpsc::Sender<update::Params>,
    converted_rx: Arc<RwLock<mpsc::Receiver<()>>>,
    update_forever_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    update_lock: Arc<Mutex<()>>,

    pub updated: Arc<RwLock<NaiveDateTime>>,
    pub period: Duration,
    pub types: Vec<Arc<Schedule>>
}
impl json::Path for Index {
    fn path(&self) -> PathBuf {
        self.path.clone()
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
            period: self.period.clone().to_std().unwrap(),
            types
        }
    }
}
impl json::Saving<MiddleIndex> for Index {}
impl Index {
    fn default(
        path: PathBuf,
        updated_tx: mpsc::Sender<update::Params>,
        converted_rx: mpsc::Receiver<()>,
    ) -> Arc<Self> {

        let dir = path.parent().unwrap_or(&path).to_path_buf();

        let this = Self {
            fetch: true,
            path,
            updated_tx,
            converted_rx: Arc::new(RwLock::new(converted_rx)),
            update_forever_handle: Arc::new(RwLock::new(None)),
            update_lock: Arc::new(Mutex::new(())),
            updated: Arc::new(RwLock::new(
                NaiveDateTime::from_timestamp(0, 0)
            )),
            period: Duration::minutes(10),
            types: vec![
                Schedule::default_ft_daily(dir.clone()),
                Schedule::default_ft_weekly(dir.clone()),
                Schedule::default_r_weekly(dir.clone()),
                Schedule::default_tchr_ft_daily(dir.clone()),
                Schedule::default_tchr_ft_weekly(dir.clone()),
                Schedule::default_tchr_r_weekly(dir)
            ]
        };

        Arc::new(this)
    }

    fn from_middle(
        middle: Arc<MiddleIndex>,
        fetch: bool,
        path: PathBuf,
        updated_tx: mpsc::Sender<update::Params>,
        converted_rx: mpsc::Receiver<()>
    ) -> Arc<Self> {

        let types = {
            let mut types = vec![];
            for sc in middle.types.iter() {
                let primary = Schedule::from_middle(
                    Arc::new(sc.clone()),
                    path.parent().map(
                        |path| path.to_path_buf()
                    ).unwrap_or(path.clone())
                );
                types.push(primary);
            }
            types
        };

        let this = Self {
            fetch,
            path,
            updated_tx,
            converted_rx: Arc::new(RwLock::new(converted_rx)),
            update_forever_handle: Arc::new(RwLock::new(None)),
            update_lock: Arc::new(Mutex::new(())),
            updated: Arc::new(RwLock::new(middle.updated.clone())),
            period: Duration::from_std(middle.period).unwrap(),
            types
        };

        Arc::new(this)
    }

    pub async fn load_or_init(
        fetch: bool,
        path: PathBuf,
        updated_tx: mpsc::Sender<update::Params>,
        converted_rx: mpsc::Receiver<()>
    ) -> SyncResult<Arc<Index>> {
        let this;

        if !path.exists() {
            this = Self::default(path, updated_tx, converted_rx);
            this.clone().save().await?;
        } else {
            this = Self::load(fetch, path, updated_tx, converted_rx).await?;
        }

        Ok(this)
    }

    pub async fn load(
        fetch: bool,
        path: PathBuf,
        updated_tx: mpsc::Sender<update::Params>,
        converted_rx: mpsc::Receiver<()>
    ) -> SyncResult<Arc<Index>> {

        let middle = MiddleIndex::load(path.clone()).await?;
        let primary = Self::from_middle(middle, fetch, path, updated_tx, converted_rx);

        Ok(primary)
    }

    pub async fn ft_daily(&self) -> Arc<Schedule> {
        self.types.iter().find(|sc| sc.sc_type == Type::FtDaily).unwrap().clone()
    }

    pub async fn ft_weekly(&self) -> Arc<Schedule> {
        self.types.iter().find(|sc| sc.sc_type == Type::FtWeekly).unwrap().clone()
    }

    pub async fn r_weekly(&self) -> Arc<Schedule> {
        self.types.iter().find(|sc| sc.sc_type == Type::RWeekly).unwrap().clone()
    }

    pub async fn tchr_ft_daily(&self) -> Arc<Schedule> {
        self.types.iter().find(|sc| sc.sc_type == Type::TchrFtDaily).unwrap().clone()
    }

    pub async fn tchr_ft_weekly(&self) -> Arc<Schedule> {
        self.types.iter().find(|sc| sc.sc_type == Type::TchrFtWeekly).unwrap().clone()
    }

    pub async fn tchr_r_weekly(&self) -> Arc<Schedule> {
        self.types.iter().find(|sc| sc.sc_type == Type::TchrRWeekly).unwrap().clone()
    }

    pub async fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            if let Err(error) = self.save().await {
                warn!("error poll saving: {:?}", error);
            }
        });
    }

    pub fn retry_period(&self) -> Duration {
        Duration::minutes(1)
    }

    pub fn update_period(&self) -> Duration {
        self.period.clone()
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

    pub async fn update_all_auto(
        self: Arc<Self>
    ) {
        self.update_all(
            update::Params {
                invoker: update::Invoker::Auto
            }
        ).await;
    }

    pub async fn update_all_manually(
        self: Arc<Self>,
        invoker: Arc<Interactor>,
    ) {
        match self.clone().update_all(
            update::Params {
                invoker: update::Invoker::Manually(invoker)
            }
        ).await {
            UpdateFinishType::Complete => {
                self.clone().abort_update_forever().await;
                self.clone().update_forever().await
            },
            UpdateFinishType::LockRelease => {
                debug!("update_all_manually() won't restart auto update loop \
                because it returned LockRelease type")
            },
        }
        
    }

    async fn update_all(
        self: Arc<Self>,
        params: update::Params
    ) -> UpdateFinishType {
        let update_lock_ref = self.update_lock.clone();

        if update_lock_ref.try_lock().is_err() {
            debug!(
                "someone tried to update while other update is still running, \
                will return after the main one finishes"
            );

            // wait for lock to be released
            let _update_lock = update_lock_ref.lock().await;

            debug!("other update finished, returning");

            // return when released
            return UpdateFinishType::LockRelease
        }

        let update_lock_ref = self.update_lock.clone();

        // lock other threads from updating
        let _update_lock = update_lock_ref.lock().await;

        debug!("updating all schedules in Index");

        let mut handles = vec![];

        self.updated_now().await;

        if self.fetch {
            for schedule in self.types.iter() {
                let schedule = schedule.clone();
    
                let handle = tokio::spawn(async move {
                    loop {
                        let bytes = schedule.refetch_until_success().await;
    
                        if let Err(error) = schedule.clone().unpack(bytes).await {
                            warn!(
                                "{} unpack error, will refetch and unpack again in {:?}: {:?}",
                                schedule.sc_type,
                                schedule.retry_period,
                                error
                            );
    
                            tokio::time::sleep(schedule.retry_period).await;
                        } else {
                            break;
                        }
                    }
                });
                handles.push(handle);
            }
    
            for handle in handles {
                handle.await.unwrap();
            }
        }

        Arc::new(self.clone().to_middle().await).poll_save();

        debug!("fetched and unpacked all successfully");

        self.clone().post_update_all(params).await;

        UpdateFinishType::Complete
    }

    async fn send_updated(self: Arc<Self>, params: update::Params) {
        self.updated_tx.send(params).await.unwrap();
        debug!("updated signal sent");
    }

    async fn await_converted(self: Arc<Self>) {
        self.converted_rx.write().await.recv().await.unwrap();
        debug!("converted signal recieved");
    }

    async fn post_update_all(self: Arc<Self>, params: update::Params) {
        self.clone().send_updated(params).await;
        self.clone().await_converted().await;
    }

    pub async fn update_forever(self: Arc<Self>) {
        let self_ref = self.clone();

        *self_ref.update_forever_handle.write().await = Some(tokio::spawn(async move {
            let mut force_until = None;

            loop {
                let next = self.clone().next_update().await;
                let mut until = if force_until.is_some() {
                    force_until.unwrap()
                } else {
                    self.clone().until_next_update().await
                };

                force_until = None;
    
                if until < Duration::zero() {
                    until = Duration::zero()
                }

                debug!("next fetch: {} (in {} secs)", next, until.num_seconds());
                tokio::time::sleep(until.to_std().unwrap()).await;

                self.clone().update_all_auto().await;
            }
        }));
    }

    pub async fn abort_update_forever(self: Arc<Self>) {
        self.update_forever_handle.read().await.as_ref().map(|handle| handle.abort());
        debug!("aborted update forever");
    }
}


#[derive(Serialize, Deserialize)]
pub struct MiddleIndex {
    #[serde(skip)]
    path: PathBuf,

    pub updated: NaiveDateTime,
    pub period: std::time::Duration,
    pub types: Vec<MiddleSchedule>
}
impl json::Path for MiddleIndex {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSaving for MiddleIndex {}
impl json::DirectLoading for MiddleIndex {}


#[derive(Debug, Clone)]
pub struct Schedule {
    root: PathBuf,
    reqwest: reqwest::Client,

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub fetch_timeout: std::time::Duration,
    pub retry_period: std::time::Duration,
    pub latest: Arc<RwLock<HashSet<PathBuf>>>,
    pub ignored: Arc<RwLock<HashSet<PathBuf>>>,
}
#[async_trait]
impl json::ToMiddle<MiddleSchedule> for Schedule {
    async fn to_middle(&self) -> MiddleSchedule {
        let mid = MiddleSchedule {
            root:          self.root.clone(),
            sc_type:       self.sc_type.clone(),
            url:           self.url.clone(),
            friendly_url:  self.friendly_url.clone(),
            fetch_timeout: self.fetch_timeout.clone(),
            retry_period:  self.retry_period.clone(),
            latest:        self.latest.read().await.clone(),
            ignored:       self.ignored.read().await.clone()
        };

        mid
    }
}
impl Schedule {
    fn from_middle(
        middle: Arc<MiddleSchedule>,
        root: PathBuf
    ) -> Arc<Self> {
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(middle.fetch_timeout.clone())
            .build()
            .unwrap();
        
        let this = Schedule {
            root,
            reqwest,
            sc_type:       middle.sc_type.clone(),
            url:           middle.url.clone(),
            friendly_url:  middle.friendly_url.clone(),
            fetch_timeout: middle.fetch_timeout,
            retry_period:  middle.retry_period,
            latest:        Arc::new(RwLock::new(middle.latest.clone())),
            ignored:       Arc::new(RwLock::new(middle.ignored.clone())),
        };

        Arc::new(this)
    }

    pub fn default_ft_daily(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(60);
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(fetch_timeout.clone())
            .build()
            .unwrap();

        let this = Schedule {
            root,
            reqwest,
            sc_type:       Type::FtDaily,
            url:           "https://docs.google.com/document/d/13FImWkHpdV_dgDCp7Py36gYPr53C-dYeUvNklkndaPA/export?format=zip".to_owned(),
            friendly_url:  "https://docs.google.com/document/d/13FImWkHpdV_dgDCp7Py36gYPr53C-dYeUvNklkndaPA".to_owned(),
            fetch_timeout,
            retry_period:  std::time::Duration::from_secs(10),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_ft_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(60);
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(fetch_timeout.clone())
            .build()
            .unwrap();

        let this = Schedule {
            root,
            reqwest,
            sc_type:       Type::FtWeekly,
            url:           "https://docs.google.com/document/d/1dHmldElsQnrdPfvRVOQnnYYG7-FWNQoEkf5a_q1CoEs/export?format=zip".to_owned(),
            friendly_url:  "https://docs.google.com/document/d/1dHmldElsQnrdPfvRVOQnnYYG7-FWNQoEkf5a_q1CoEs".to_owned(),
            fetch_timeout,
            retry_period:  std::time::Duration::from_secs(10),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_r_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(60);
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(fetch_timeout.clone())
            .build()
            .unwrap();

        let this = Schedule {
            root,
            reqwest,
            sc_type:       Type::RWeekly,
            url:           "https://docs.google.com/spreadsheets/d/1khl9YgQ9cAwFYdnHUsr0jfvCr2cea3WhTjBKfILdEeE/export?format=zip".to_owned(),
            friendly_url:  "https://docs.google.com/spreadsheets/d/1khl9YgQ9cAwFYdnHUsr0jfvCr2cea3WhTjBKfILdEeE".to_owned(),
            fetch_timeout,
            retry_period:  std::time::Duration::from_secs(10),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };
        
        Arc::new(this)
    }

    pub fn default_tchr_ft_daily(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(60);
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(fetch_timeout.clone())
            .build()
            .unwrap();

        let this = Schedule {
            root,
            reqwest,
            sc_type:       Type::TchrFtDaily,
            url:           "https://docs.google.com/document/d/1gEP_8FhNRWQuKSLTnynqJWFcCCpyDaetu-fqw_gvFjs/export?format=zip".to_owned(),
            friendly_url:  "https://docs.google.com/document/d/1gEP_8FhNRWQuKSLTnynqJWFcCCpyDaetu-fqw_gvFjs".to_owned(),
            fetch_timeout,
            retry_period:  std::time::Duration::from_secs(10),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_tchr_ft_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(60);
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(fetch_timeout.clone())
            .build()
            .unwrap();

        let this = Schedule {
            root,
            reqwest,
            sc_type:       Type::TchrFtWeekly,
            url:           "https://docs.google.com/document/d/16JJ-auyHCNdNNOF71bkkIe0o089NjCKp6DgcDrMgn9I/export?format=zip".to_owned(),
            friendly_url:  "https://docs.google.com/document/d/16JJ-auyHCNdNNOF71bkkIe0o089NjCKp6DgcDrMgn9I".to_owned(),
            fetch_timeout,
            retry_period:  std::time::Duration::from_secs(10),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_tchr_r_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(60);
        let reqwest = reqwest::ClientBuilder::new()
            .timeout(fetch_timeout.clone())
            .build()
            .unwrap();

        let this = Schedule {
            root,
            reqwest,
            sc_type:       Type::TchrRWeekly,
            url:           "https://docs.google.com/spreadsheets/d/1vX4TCTZTZtg8b1LMC_tbYFZw-7tD4JQ2UwxEvagTb1k/export?format=zip".to_owned(),
            friendly_url:  "https://docs.google.com/spreadsheets/d/1vX4TCTZTZtg8b1LMC_tbYFZw-7tD4JQ2UwxEvagTb1k".to_owned(),
            fetch_timeout,
            retry_period:  std::time::Duration::from_secs(10),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };
        
        Arc::new(this)
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }

    pub async fn get_latest(&self) -> SyncResult<HashSet<PathBuf>> {
        let dir = self.dir();

        let paths = match self.sc_type {
            Type::FtDaily | Type::TchrFtDaily | Type::FtWeekly | Type::TchrFtWeekly => {
                let mut hs = HashSet::new();
                if let Some(path) = fulltime::latest(&dir).await.unwrap() {
                    hs.insert(path);
                }
                hs
            },
            Type::RWeekly => remote::latest(&dir, super::Mode::Groups).await.unwrap(),
            Type::TchrRWeekly => remote::latest(&dir, super::Mode::Teachers).await.unwrap(),
        };

        *self.latest.write().await = paths.clone();

        Ok(paths)
    }

    pub async fn has_latest(&self) -> bool {
        !self.latest.read().await.is_empty()
    }

    pub async fn get_ignored(&self) -> tokio::io::Result<Option<HashSet<PathBuf>>> {
        let latest = self.latest.read().await;
        if latest.is_empty() {
            return Ok(None)
        }

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
        let resp = self.reqwest.get(&self.url).send().await?;
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
            warn!(
                "refetching {} because of error {:?}",
                self.sc_type,
                error
            );

            tokio::time::sleep(self.retry_period).await;
            return self.refetch_until_success().await
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
    #[serde(skip)]
    root: PathBuf,

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub fetch_timeout: std::time::Duration,
    pub retry_period: std::time::Duration,
    pub latest: HashSet<PathBuf>,
    pub ignored: HashSet<PathBuf>,
}
impl MiddleSchedule {
    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }
}
