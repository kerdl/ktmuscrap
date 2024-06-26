use futures_lite::AsyncReadExt;
use log::{debug, warn};
use chrono::{NaiveDateTime, DateTime, Utc, Duration};
use async_zip::tokio::read::seek::ZipFileReader;
use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use reqwest;
use actix_web::web::Bytes;
use tokio::{
    io::AsyncWriteExt, sync::{
        mpsc, Mutex, RwLock
    }, task::JoinHandle
};
use tokio_util::compat::TokioAsyncWriteCompatExt;
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
                DateTime::from_timestamp(0, 0).unwrap().naive_utc()
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
        debug!("staring auto update_all");
        self.update_all(
            update::Params {
                invoker: update::Invoker::Auto
            }
        ).await;
        debug!("finished auto update_all");
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
        debug!("converted signal received");
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
        let fetch_timeout = std::time::Duration::from_secs(90);
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
            retry_period:  std::time::Duration::from_secs(2),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_ft_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(90);
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
            retry_period:  std::time::Duration::from_secs(2),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_r_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(90);
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
            retry_period:  std::time::Duration::from_secs(2),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };
        
        Arc::new(this)
    }

    pub fn default_tchr_ft_daily(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(90);
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
            retry_period:  std::time::Duration::from_secs(2),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_tchr_ft_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(90);
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
            retry_period:  std::time::Duration::from_secs(2),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };

        Arc::new(this)
    }

    pub fn default_tchr_r_weekly(root: PathBuf) -> Arc<Schedule> {
        let fetch_timeout = std::time::Duration::from_secs(90);
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
            retry_period:  std::time::Duration::from_secs(2),
            latest:        Arc::new(RwLock::new(HashSet::new())),
            ignored:       Arc::new(RwLock::new(HashSet::new())),
        };
        
        Arc::new(this)
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(self.sc_type.to_string())
    }

    pub async fn get_latest(&self, files: &Vec<update::File>) -> SyncResult<HashSet<PathBuf>> {
        let paths = match self.sc_type {
            Type::FtDaily | Type::TchrFtDaily | Type::FtWeekly | Type::TchrFtWeekly => {
                let mut hs = HashSet::new();
                if let Some(path) = fulltime::latest(&files).await {
                    hs.insert(path);
                }
                hs
            },
            Type::RWeekly => remote::latest(&files, super::Mode::Groups).await.unwrap(),
            Type::TchrRWeekly => remote::latest(&files, super::Mode::Teachers).await.unwrap(),
        };

        *self.latest.write().await = paths.clone();

        Ok(paths)
    }

    pub async fn has_latest(&self) -> bool {
        !self.latest.read().await.is_empty()
    }

    pub async fn get_ignored(&self, files: &Vec<update::File>) -> tokio::io::Result<Option<HashSet<PathBuf>>> {
        let latest = self.latest.read().await;
        if latest.is_empty() {
            return Ok(None)
        }

        let ignored = ignored::except_difference(
            &HashSet::from_iter(files.iter().map(|file| file.path.clone())),
            &latest,
        );

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
        let bytes = resp.bytes().await;
        bytes
    }

    pub async fn fetch_after(&self, after: Duration) -> Result<Bytes, reqwest::Error> {
        tokio::time::sleep(after.to_std().unwrap()).await;
        self.fetch().await
    }

    pub async fn refetch_until_success(&self) -> Bytes {
        loop {
            debug!("fetching {}", self.sc_type);
            let fetch_result = self.fetch().await;
    
            if let Err(error) = &fetch_result {
                warn!(
                    "refetching {} because of error {:?}",
                    self.sc_type,
                    error
                );
    
                tokio::time::sleep(self.retry_period).await;
                continue;
            }
            
            debug!("fetching {} success", self.sc_type);
            return fetch_result.unwrap()
        };
    }

    pub async fn unpack(self: Arc<Self>, bytes: Bytes) -> Result<(), error::UnpackError> {
        let dir = self.dir();
        let sc_type = self.sc_type.clone();

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

        let cursor = Cursor::new(bytes);
        debug!("parsing {:?} archive", sc_type);
        let archive_result = ZipFileReader::with_tokio(cursor).await;
        debug!("parsed {:?} archive", sc_type);

        if let Err(error) = archive_result {
            return Err(error::UnpackError::Zip(error));
        }

        let mut archive = archive_result.unwrap();
        let mut files: Vec<update::File> = vec![];

        debug!("extracting {:?} archive", sc_type);
        for index in 0..archive.file().entries().len() {
            let entry = archive.file().entries().get(index).unwrap();
            let filename_pathbuf = fs::path::sanitize(entry.filename().as_str().unwrap());
            let path = dir.join(filename_pathbuf);
            let entry_is_dir = entry.dir().unwrap();

            let entry_reader = archive.reader_without_entry(index).await;
            if let Err(err) = entry_reader {
                warn!("entry reader error: {:?}", err);
                continue;
            }
            let mut entry_reader = entry_reader.unwrap();

            if entry_is_dir {
                // The directory may have been created if iteration is out of order.
                if !path.exists() {
                    let result = tokio::fs::create_dir_all(&path).await;
                    if let Err(err) = result {
                        warn!("failed to create extracted directory: {:?}", err);
                        continue;
                    }
                }
            } else {
                // Creates parent directories. They may not exist if iteration is out of order
                // or the archive does not contain directory entries.
                let parent = path.parent();
                if parent.is_none() {
                    warn!("a file entry should have parent directories");
                    continue;
                }
                let parent = parent.unwrap();
                if !parent.is_dir() {
                    let result = tokio::fs::create_dir_all(parent).await;
                    if let Err(err) = result {
                        warn!("failed to create parent directories: {:?}", err);
                        continue;
                    }
                }

                let mut buf = vec![];
                let copy_result = entry_reader.read_to_end(&mut buf).await;

                if let Err(err) = copy_result {
                    warn!("failed to copy to extracted file: {:?}", err);
                    continue;
                }

                let file = update::File {
                    path,
                    bytes: buf.into()
                };
                files.push(file);
    
                // Closes the file and manipulates its metadata here if you wish to preserve its metadata from the archive.
            }
        }
        debug!("extracted {:?} archive", sc_type);

        self.post_unpack(files).await;

        Ok(())
    }

    async fn post_unpack(self: Arc<Self>, mut files: Vec<update::File>) {
        files.retain(|file| file.path.extension() == Some(std::ffi::OsStr::new("html")));

        let latest = self.purge_ignored(files.clone()).await;
        self.get_latest(&latest).await.unwrap();
        self.get_ignored(&files).await.unwrap();

        let latest = self.latest.read().await;

        for latest_path in latest.iter() {
            let writer = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&latest_path)
                .await;
            if let Err(err) = writer {
                warn!("failed to create extracted file: {:?}", err);
                continue;
            }
            let mut writer = writer.unwrap();

            let file = files.iter().find(|file| &file.path == latest_path).unwrap();
 
            if let Err(err) = writer.write(&file.bytes[..]).await {
                warn!("failed to write file: {:?}", err);
                continue;
            }
        }
    }

    pub async fn purge_ignored(&self, mut files: Vec<update::File>) -> Vec<update::File> {
        let ignored = self.ignored.read().await;
        files.retain(|file| !ignored.contains(&file.path));
        files
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
