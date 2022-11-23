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
        Loading,
        DirectSaving,
        DirectLoading
    }, schedule::Interactor},
    fs, parse
};
use super::{
    super::update,
    Type,
    ignored,
    fulltime,
    remote,
    error,
};


#[derive(Debug)]
pub struct Index {
    path: PathBuf,
    updated_tx: mpsc::Sender<update::Params>,
    converted_rx: Arc<RwLock<mpsc::Receiver<()>>>,
    update_forever_handle: Arc<RwLock<Option<JoinHandle<()>>>>,

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
            path,
            updated_tx,
            converted_rx: Arc::new(RwLock::new(converted_rx)),
            update_forever_handle: Arc::new(RwLock::new(None)),
            updated: Arc::new(RwLock::new(
                NaiveDateTime::from_timestamp(0, 0)
            )),
            period: Duration::minutes(10),
            types: vec![
                Schedule::default_ft_daily(dir.clone()),
                Schedule::default_ft_weekly(dir.clone()),
                Schedule::default_r_weekly(dir)
            ]
        };

        Arc::new(this)
    }

    fn from_middle(
        middle: Arc<MiddleIndex>,
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
            path,
            updated_tx,
            converted_rx: Arc::new(RwLock::new(converted_rx)),
            update_forever_handle: Arc::new(RwLock::new(None)),
            updated: Arc::new(RwLock::new(middle.updated.clone())),
            period: Duration::from_std(middle.period).unwrap(),
            types
        };

        Arc::new(this)
    }

    pub async fn load_or_init(
        path: PathBuf,
        updated_tx: mpsc::Sender<update::Params>,
        converted_rx: mpsc::Receiver<()>
    ) -> SyncResult<Arc<Index>> {
        let this;

        if !path.exists() {
            this = Self::default(path, updated_tx, converted_rx);
            this.clone().save().await?;
        } else {
            this = Self::load(path, updated_tx, converted_rx).await?;
        }

        Ok(this)
    }

    pub async fn load(
        path: PathBuf,
        updated_tx: mpsc::Sender<update::Params>,
        converted_rx: mpsc::Receiver<()>
    ) -> SyncResult<Arc<Index>> {

        let middle = MiddleIndex::load(path.clone()).await?;
        let primary = Self::from_middle(middle, path, updated_tx, converted_rx);

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
        ).await
    }

    pub async fn update_all_manually(
        self: Arc<Self>,
        invoker: Arc<Interactor>,
    ) {
        self.clone().abort_update_forever().await;
        self.clone().update_all(
            update::Params {
                invoker: update::Invoker::Manually(invoker)
            }
        ).await;
        self.clone().update_forever().await;
    }

    async fn update_all(
        self: Arc<Self>,
        params: update::Params
    ) {

        debug!("updating all schedules in Index");

        let mut handles = vec![];

        self.updated_now().await;

        for schedule in self.types.iter() {
            let schedule = schedule.clone();

            let handle = tokio::spawn(async move {
                loop {
                    let bytes = schedule.refetch_until_success().await;

                    if let Err(error) = schedule.clone().unpack(bytes).await {
                        warn!(
                            "{} unpack error, will refetch and unpack again in {}: {:?}",
                            schedule.sc_type,
                            schedule.retry_period(),
                            error
                        );

                        tokio::time::sleep(schedule.retry_period().to_std().unwrap()).await;
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

        Arc::new(self.clone().to_middle().await).poll_save();

        debug!("fetched and unpacked all successfully");

        self.clone().post_update_all(params).await;
    }

    async fn post_update_all(self: Arc<Self>, params: update::Params) {
        self.updated_tx.send(params).await.unwrap();
        debug!("updated signal sent");
        self.converted_rx.write().await.recv().await.unwrap();
        debug!("converted signal recieved");
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

    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Arc<RwLock<Option<PathBuf>>>,
    pub ignored: Arc<RwLock<HashSet<PathBuf>>>,
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
    fn from_middle(
        middle: Arc<MiddleSchedule>,
        root: PathBuf
    ) -> Arc<Self> {

        let this = Schedule {
            root,
            sc_type:      middle.sc_type.clone(),
            url:          middle.url.clone(),
            friendly_url: middle.friendly_url.clone(),
            latest:       Arc::new(RwLock::new(middle.latest.clone())),
            ignored:      Arc::new(RwLock::new(middle.ignored.clone())),
        };

        Arc::new(this)
    }

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

    pub fn retry_period(&self) -> Duration {
        Duration::minutes(1)
    }

    pub async fn get_latest(&self) -> SyncResult<Option<PathBuf>> {
        let dir = self.dir();

        let path = match self.sc_type {
            Type::FtDaily  => fulltime::latest(&dir).await.unwrap(),
            Type::FtWeekly => fulltime::latest(&dir).await.unwrap(),
            Type::RWeekly  => remote::latest(&dir).await.unwrap()
        };

        *self.latest.write().await = path.clone();

        Ok(path)
    }

    pub async fn has_latest(&self) -> bool {
        self.latest.read().await.is_some()
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
            warn!(
                "refetching {} because of error {:?}",
                self.sc_type,
                error
            );

            tokio::time::sleep(self.retry_period().to_std().unwrap()).await;
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
    pub latest: Option<PathBuf>,
    pub ignored: HashSet<PathBuf>,
}
impl MiddleSchedule {
    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }
}
