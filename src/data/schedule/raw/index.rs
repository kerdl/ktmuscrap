use chrono::NaiveDateTime;
use ::zip::ZipArchive;
use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use hex;
use reqwest;
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{io::Cursor, path::PathBuf, sync::Arc};

use crate::{
    SyncResult,
    DATA_PATH,
    REMOTE_INDEX_PATH,
    data::json,
    fs
};
use super::{
    Type,
    fulltime,
    remote
};


pub struct Index {
    path: PathBuf,
    pub updated: NaiveDateTime,
    pub types: Vec<Schedule>
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
            updated: NaiveDateTime::from_timestamp(0, 0),
            types: vec![
                Schedule::default_ft_daily(),
                Schedule::default_ft_weekly(),
                Schedule::default_r_weekly()
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
                types.push((*primary).clone());
            }
            types
        };

        let this = Self {
            path: middle.path.clone(),
            updated: middle.updated.clone(),
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
            updated: self.updated.clone(),
            types
        }
    }
}
impl json::SavingLoading<MiddleIndex> for Index {}

#[derive(Serialize, Deserialize)]
pub struct MiddleIndex {
    #[serde(skip)]
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
    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Arc<RwLock<Option<PathBuf>>>,
    pub sha256: Arc<RwLock<Option<String>>>,
}
impl json::FromMiddle<MiddleSchedule> for Schedule {
    fn from_middle(middle: Arc<MiddleSchedule>) -> Arc<Self> {
        let this = Schedule {
            sc_type: middle.sc_type.clone(),
            url: middle.url.clone(),
            friendly_url: middle.friendly_url.clone(),
            latest: Arc::new(RwLock::new(middle.latest.clone())),
            sha256: Arc::new(RwLock::new(middle.sha256.clone()))
        };

        Arc::new(this)
    }
}
#[async_trait]
impl json::ToMiddle<MiddleSchedule> for Schedule {
    async fn to_middle(&self) -> MiddleSchedule {
        let mid = MiddleSchedule {
            sc_type: self.sc_type.clone(),
            url: self.url.clone(),
            friendly_url: self.friendly_url.clone(),
            latest: self.latest.read().await.clone(),
            sha256: self.sha256.read().await.clone()
        };

        mid
    }
}
impl Schedule {
    pub fn default_ft_daily() -> Schedule {
        Schedule {
            sc_type:      Type::FtDaily,
            url:          "https://docs.google.com/document/d/1gsE6aikIQ1umKSQWVnyn3_59mnGQQU8O/export?format=zip".to_owned(),
            friendly_url: "https://docs.google.com/document/d/1gsE6aikIQ1umKSQWVnyn3_59mnGQQU8O".to_owned(),
            latest:       Arc::new(RwLock::new(None)),
            sha256:       Arc::new(RwLock::new(None)),
        }
    }

    pub fn default_ft_weekly() -> Schedule {
        Schedule {
            sc_type:      Type::FtWeekly,
            url:          "https://docs.google.com/document/d/1FH4ctIgRX1fWjIPoboXWieEYVMDYSlg4/export?format=zip".to_owned(),
            friendly_url: "https://docs.google.com/document/d/1FH4ctIgRX1fWjIPoboXWieEYVMDYSlg4".to_owned(),
            latest:       Arc::new(RwLock::new(None)),
            sha256:       Arc::new(RwLock::new(None)),
        }
    }

    pub fn default_r_weekly() -> Schedule {
        Schedule {
            sc_type:      Type::RWeekly,
            url:          "https://docs.google.com/spreadsheets/d/1SWv7ARLLC6S_FjIzzhUz0kzGCdG53t9xL68VPoiYlnA/export?format=zip".to_owned(),
            friendly_url: "https://docs.google.com/spreadsheets/d/1SWv7ARLLC6S_FjIzzhUz0kzGCdG53t9xL68VPoiYlnA".to_owned(),
            latest:       Arc::new(RwLock::new(None)),
            sha256:       Arc::new(RwLock::new(None)),
        }
    }

    pub fn dir(&self) -> PathBuf {
        DATA_PATH.join(self.sc_type.to_string())
    }

    pub async fn fetch(&self) -> Result<Bytes, reqwest::Error> {
        let resp = reqwest::get(&self.url).await?;

        resp.bytes().await
    }

    pub async fn unpack(&self, bytes: Bytes) -> SyncResult<()> {
        if self.dir().exists() {
            fs::remove::all_except(
                &self.dir(),
                &REMOTE_INDEX_PATH
            ).await?;
        }

        let path = self.dir();

        tokio::task::spawn_blocking(move || {
            let cursor = Cursor::new(bytes);
            let mut archive = ZipArchive::new(cursor)?;
            archive.extract(path)?;
    
            Ok(())
        }).await?
    }

    pub async fn get_latest(&self) -> SyncResult<Option<PathBuf>> {
        let latest = match self.sc_type {
            Type::FtDaily => fulltime::latest(&self.dir()).await?,
            Type::FtWeekly => fulltime::latest(&self.dir()).await?,
            Type::RWeekly => remote::latest(&self.dir()).await?
        };

        *self.latest.write().await = latest.clone();

        Ok(latest)
    }

    pub async fn get_sha256(&self) -> SyncResult<Option<String>> {
        let latest_path = self.get_latest().await?;
        if latest_path.is_none() {
            return Ok(None)
        }
        let latest_path = latest_path.unwrap();
    
        let latest_html = tokio::fs::read_to_string(latest_path).await?;

        let mut hasher = Sha256::default();
        hasher.update(latest_html.as_bytes());
        let bytes_hash = hasher.finalize();

        let string_hash = hex::encode(bytes_hash);

        *self.sha256.write().await = Some(string_hash.clone());

        Ok(Some(string_hash))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MiddleSchedule {
    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Option<PathBuf>,
    pub sha256: Option<String>,
}
impl json::Path for MiddleSchedule {
    fn path(&self) -> PathBuf {
        self.dir().join("index.json")
    }
}
impl json::DirectSavingLoading for MiddleSchedule {}
impl MiddleSchedule {
    pub fn dir(&self) -> PathBuf {
        DATA_PATH.join(self.sc_type.to_string())
    }
}