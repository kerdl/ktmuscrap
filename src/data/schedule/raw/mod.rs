mod index;
mod zip;
pub mod container;
pub mod table;
pub mod fulltime;
pub mod remote;

pub use self::zip::Zip;
pub use container::{
    Container,
    Schedule as ScheduleContainer
};

use lazy_static::lazy_static;
use ::zip::ZipArchive;
use serde_derive::{Serialize, Deserialize};
use strum_macros::{EnumString, Display};
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
    fs
};


lazy_static! {
    static ref FT_DAILY: Schedule = Schedule {
        sc_type:      Type::FtDaily,
        url:          "https://docs.google.com/document/d/1gsE6aikIQ1umKSQWVnyn3_59mnGQQU8O/export?format=zip".to_owned(),
        friendly_url: "https://docs.google.com/document/d/1gsE6aikIQ1umKSQWVnyn3_59mnGQQU8O".to_owned(),
        latest:       Arc::new(RwLock::new(None)),
        sha256:       Arc::new(RwLock::new(None)),
    };
    static ref FT_WEEKLY: Schedule = Schedule {
        sc_type:      Type::FtWeekly,
        url:          "https://docs.google.com/document/d/1FH4ctIgRX1fWjIPoboXWieEYVMDYSlg4/export?format=zip".to_owned(),
        friendly_url: "https://docs.google.com/document/d/1FH4ctIgRX1fWjIPoboXWieEYVMDYSlg4".to_owned(),
        latest:       Arc::new(RwLock::new(None)),
        sha256:       Arc::new(RwLock::new(None)),
    };
    static ref R_WEEKLY: Schedule = Schedule {
        sc_type:      Type::RWeekly,
        url:          "https://docs.google.com/spreadsheets/d/1SWv7ARLLC6S_FjIzzhUz0kzGCdG53t9xL68VPoiYlnA/export?format=zip".to_owned(),
        friendly_url: "https://docs.google.com/spreadsheets/d/1SWv7ARLLC6S_FjIzzhUz0kzGCdG53t9xL68VPoiYlnA".to_owned(),
        latest:       Arc::new(RwLock::new(None)),
        sha256:       Arc::new(RwLock::new(None)),
    };
}


#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    EnumString,
    Hash
)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    FtDaily,
    FtWeekly,
    RWeekly
}

#[derive(Debug)]
pub struct Schedule {
    pub sc_type: Type,
    pub url: String,
    pub friendly_url: String,
    pub latest: Arc<RwLock<Option<PathBuf>>>,
    pub sha256: Arc<RwLock<Option<String>>>,
}
impl Schedule {
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
                &DATA_PATH,
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