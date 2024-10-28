use log::{debug, warn};
use chrono::{NaiveDateTime, DateTime, Utc, Duration};
use serde_derive::{Serialize, Deserialize};
use reqwest;
use tokio::{
    io::AsyncWriteExt, sync::{mpsc, Mutex, RwLock}, task::JoinHandle
};
use std::{
    path::PathBuf,
    sync::Arc
};
use crate::{
    regexes,
    SyncResult,
    data::{
        json::{
            self,
            ToMiddle,
            Saving,
            DirectSaving,
            DirectLoading
        },
        schedule::{
            raw::{Kind, error},
            File
        },
    }
};

enum UpdateFinishType {
    Complete,
    LockRelease
}

#[derive(Debug)]
pub struct Index {
    path: PathBuf,
    updated_tx: mpsc::Sender<Vec<PathHolder>>,
    converted_rx: Arc<RwLock<mpsc::Receiver<()>>>,
    update_forever_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    update_lock: Arc<Mutex<()>>,

    pub fetch: bool,
    pub updated: Arc<RwLock<NaiveDateTime>>,
    pub period: Duration,
    pub ignored: Vec<String>,
    pub types: Vec<Arc<Schedule>>
}
impl json::Path for Index {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl Index {
    fn default(
        path: PathBuf,
        updated_tx: mpsc::Sender<Vec<PathHolder>>,
        converted_rx: mpsc::Receiver<()>,
    ) -> Arc<Self> {
        let this = Self {
            path,
            updated_tx,
            converted_rx: Arc::new(RwLock::new(converted_rx)),
            update_forever_handle: Arc::new(RwLock::new(None)),
            update_lock: Arc::new(Mutex::new(())),
            fetch: true,
            ignored: vec![],
            updated: Arc::new(RwLock::new(
                DateTime::from_timestamp(0, 0).unwrap().naive_utc()
            )),
            period: Duration::minutes(10),
            types: vec![]
        };

        Arc::new(this)
    }

    fn from_middle(
        middle: Arc<MiddleIndex>,
        path: PathBuf,
        updated_tx: mpsc::Sender<Vec<PathHolder>>,
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
            update_lock: Arc::new(Mutex::new(())),
            fetch: middle.fetch,
            ignored: middle.ignored.clone(),
            updated: Arc::new(RwLock::new(middle.updated)),
            period: Duration::from_std(middle.period).unwrap(),
            types
        };

        Arc::new(this)
    }

    pub async fn load_or_init(
        path: PathBuf,
        updated_tx: mpsc::Sender<Vec<PathHolder>>,
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
        updated_tx: mpsc::Sender<Vec<PathHolder>>,
        converted_rx: mpsc::Receiver<()>
    ) -> SyncResult<Arc<Index>> {
        let middle = MiddleIndex::load(path.clone()).await?;
        let primary = Self::from_middle(middle, path, updated_tx, converted_rx);

        Ok(primary)
    }

    pub async fn groups(&self) -> Vec<Arc<Schedule>> {
        self.types.iter().filter(|sc| sc.kind == Kind::Groups).cloned().collect::<Vec<Arc<Schedule>>>()
    }

    pub async fn teachers(&self) -> Vec<Arc<Schedule>> {
        self.types.iter().filter(|sc| sc.kind == Kind::Teachers).cloned().collect::<Vec<Arc<Schedule>>>()
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
        self.period
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

    async fn update_all(self: Arc<Self>) -> UpdateFinishType {
        let mut paths = vec![];
        let update_lock_ref = self.update_lock.clone();

        if update_lock_ref.try_lock().is_err() {
            debug!(
                "someone tried to update while the other update is still running, \
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
                if self.ignored.contains(&schedule.name) {
                    continue;
                }

                let schedule = schedule.clone();
    
                let handle = tokio::spawn(async move {
                    let paths;
                    loop {
                        let string = schedule.refetch_until_success().await;
                        let unpack_result = schedule.clone().steal_sheets(string).await;

                        if let Ok(collected_paths) = unpack_result {
                            paths = collected_paths;
                            break;
                        } else if let Err(error) = unpack_result {
                            warn!(
                                "{} unpack error, will refetch and unpack again in {:?}: {:?}",
                                schedule.kind,
                                schedule.retry_period,
                                error
                            );
    
                            tokio::time::sleep(schedule.retry_period).await;
                        }
                    }
                    PathHolder {
                        paths,
                        name: schedule.name.clone(),
                        kind: schedule.kind
                    }
                });
                handles.push(handle);
            }
    
            for handle in handles {
                let path_index = handle.await.unwrap();
                paths.push(path_index);
            }

            debug!("fetched and unpacked all successfully");
        } else {
            debug!("fetching is disabled, update bypassed");
        }

        Arc::new(self.clone().to_middle().await).poll_save();
        
        self.clone().signal_updated(paths).await;
        self.clone().await_converted().await;

        UpdateFinishType::Complete
    }

    async fn signal_updated(self: Arc<Self>, paths: Vec<PathHolder>) {
        self.updated_tx.send(paths).await.unwrap();
        debug!("updated signal sent");
    }

    async fn await_converted(self: Arc<Self>) {
        self.converted_rx.write().await.recv().await.unwrap();
        debug!("converted signal received");
    }

    pub async fn update_forever(self: Arc<Self>) {
        let self_ref = self.clone();

        *self_ref.update_forever_handle.write().await = Some(tokio::spawn(async move {
            loop {
                let next = self.clone().next_update().await;
                let mut until = self.clone().until_next_update().await;
                let zero_dur = Duration::zero();
    
                if until < zero_dur {
                    until = zero_dur;
                }

                debug!("next fetch: {} (in {} secs)", next, until.num_seconds());
                tokio::time::sleep(until.to_std().unwrap()).await;
                self.clone().update_all().await;
            }
        }));
    }

    pub async fn abort_update_forever(self: Arc<Self>) {
        if let Some(handle) = self.update_forever_handle.read().await.as_ref() {
            handle.abort()
        };
        debug!("aborted update forever");
    }
}
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
            fetch: self.fetch,
            ignored: self.ignored.clone(),
            updated: *self.updated.read().await,
            period: self.period.clone().to_std().unwrap(),
            types
        }
    }
}
impl json::Saving<MiddleIndex> for Index {}


#[derive(Serialize, Deserialize)]
pub struct MiddleIndex {
    #[serde(skip)]
    path: PathBuf,

    pub fetch: bool,
    pub updated: NaiveDateTime,
    pub period: std::time::Duration,
    pub ignored: Vec<String>,
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

    pub kind: Kind,
    pub name: String,
    pub url: String,
    pub fetch_timeout: std::time::Duration,
    pub retry_period: std::time::Duration,
}

impl json::ToMiddle<MiddleSchedule> for Schedule {
    async fn to_middle(&self) -> MiddleSchedule {
        MiddleSchedule {
            root: self.root.clone(),
            kind: self.kind,
            name: self.name.clone(),
            url: self.url.clone(),
            fetch_timeout: self.fetch_timeout,
            retry_period: self.retry_period,
        }
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
            kind: middle.kind,
            name: middle.name.clone(),
            url: middle.url.clone(),
            fetch_timeout: middle.fetch_timeout,
            retry_period: middle.retry_period,
        };

        Arc::new(this)
    }

    pub fn dir(&self) -> PathBuf {
        self.root.join(self.name.clone())
    }

    pub async fn fetch_custom(&self, url: &str) -> Result<String, reqwest::Error> {
        let resp = self.reqwest.get(url).send().await?.error_for_status()?;
        resp.text().await
    }

    pub async fn fetch(&self) -> Result<String, reqwest::Error> {
        self.fetch_custom(&self.url).await
    }

    pub async fn fetch_after(&self, after: Duration) -> Result<String, reqwest::Error> {
        tokio::time::sleep(after.to_std().unwrap()).await;
        self.fetch().await
    }

    pub async fn refetch_until_success_custom(
        &self,
        name: &str,
        url: &str,
        retry_period: std::time::Duration
    ) -> String {
        loop {
            debug!("fetching {} ({})", name, url);
            let fetch_result = self.fetch_custom(url).await;
    
            if let Err(error) = &fetch_result {
                warn!(
                    "refetching {} because of error {:?}",
                    name,
                    error
                );
    
                tokio::time::sleep(retry_period).await;
                continue;
            }
            
            debug!("fetching {} success", name);
            return fetch_result.unwrap()
        };
    }

    pub async fn refetch_until_success(&self) -> String {
        self.refetch_until_success_custom(
            &self.name,
            &self.url,
            self.retry_period
        ).await
    }

    pub async fn steal_sheets(self: Arc<Self>, string: String) -> Result<Vec<PathBuf>, error::UnpackError> {
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

        let mut sheet_urls: Vec<String> = vec![];
        let mut handles: Vec<JoinHandle<File>> = vec![];
        let mut files: Vec<File> = vec![];

        for quoted_string in regexes().strings.find_iter(&string) {
            let quoted_string = quoted_string.as_str()
                .replace(r#"\/"#, r#"/"#);

            let mut quoted_string_chars = quoted_string.chars();
            quoted_string_chars.next();
            quoted_string_chars.next_back();
            let pure_string = quoted_string_chars.as_str();

            if !pure_string.starts_with("http") || !pure_string.contains("gid=") {
                continue;
            }

            sheet_urls.push(pure_string.to_string());
        }

        for (idx, sheet_url) in sheet_urls.into_iter().enumerate() {
            let self_ref = self.clone();
            let handle = tokio::spawn(async move {
                let string = self_ref.refetch_until_success_custom(
                    &format!("{}/{} ({})", self_ref.name, idx.to_string(), sheet_url),
                    &sheet_url,
                    self_ref.retry_period
                ).await;

                File {
                    path: self_ref.dir().join(format!("{}.html", idx)),
                    string
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            files.push(handle.await.unwrap());
        }

        let unpacked_paths = self.post_unpack(files).await;

        Ok(unpacked_paths)
    }

    async fn post_unpack(self: Arc<Self>, mut files: Vec<File>) -> Vec<PathBuf> {
        let html_ext = std::ffi::OsStr::new("html");
        files.retain(|file| file.path.extension() == Some(html_ext));

        for file in files.iter() {
            let writer = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&file.path)
                .await;
            if let Err(err) = writer {
                warn!("failed to create extracted file: {:?}", err);
                continue;
            }
            let mut writer = writer.unwrap();
 
            if let Err(err) = writer.write_all(file.string.as_bytes()).await {
                warn!("failed to write file: {:?}", err);
                continue;
            }
        }

        files.into_iter().map(|file| file.path).collect()
    }
}

#[derive(Debug)]
pub struct PathHolder {
    pub paths: Vec<PathBuf>,
    pub name: String,
    pub kind: Kind
}

/// # Stores last converted raw schedules
#[derive(Debug)]
pub struct PathContainer {
    pub list: Arc<RwLock<Vec<PathHolder>>>
}
impl PathContainer {
    pub fn default() -> Arc<Self> {
        let this = Self {
            list: Arc::new(RwLock::new(vec![]))
        };
        Arc::new(this)
    }

    /// # Check if all `names` are present
    pub async fn is_complete(&self, reference: &[String]) -> bool {
        self.list.read().await.iter().all(|sc| reference.contains(&sc.name))
    }

    /// # Get `names` that are not present
    pub async fn missing_names(&self, reference: &[String]) -> Vec<String> {
        self.list.read().await.iter()
            .filter(|sc| !reference.contains(&sc.name))
            .map(|sc| sc.name.clone())
            .collect()
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct MiddleSchedule {
    #[serde(skip)]
    root: PathBuf,

    pub kind: Kind,
    pub name: String,
    pub url: String,
    pub fetch_timeout: std::time::Duration,
    pub retry_period: std::time::Duration,
}
impl MiddleSchedule {
    pub fn example() -> Self {
        Self {
            root: std::path::PathBuf::new(),
            kind: crate::data::schedule::raw::Kind::Groups,
            name: "Containing folder name".to_string(),
            url: "https://docs.google.com/document/d/13FImWkHpdV_dgDCp7Py36gYPr53C-dYeUvNklkndaPA/export?format=zip".to_string(),
            fetch_timeout: std::time::Duration::from_secs(90),
            retry_period: std::time::Duration::from_secs(2)
        }
    }
    pub fn dir(&self) -> PathBuf {
        self.root.join(self.kind.to_string())
    }
}
