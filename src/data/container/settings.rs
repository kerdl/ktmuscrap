use std::{sync::Arc, path::PathBuf};
use serde_derive::{Serialize, Deserialize};
use crate::{SyncResult, data::json::{self, DirectLoading, DirectSaving}};


#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub address: String
}
impl Server {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8080".to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Parsing {
    pub fulltime_color: String,
    pub remote_color: String
}
impl Parsing {
    fn default() -> Self {
        Self {
            fulltime_color: "#fde9d9".to_string(),
            remote_color: "#c6d9f0".to_string()
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(skip)]
    path: PathBuf,
    pub server: Server,
    pub parsing: Parsing
}
impl Settings {
    fn default(path: PathBuf) -> Arc<Self> {
        let this = Self {
            path,
            server: Server::default(),
            parsing: Parsing::default()
        };

        Arc::new(this)
    }

    pub async fn load_or_init(path: PathBuf) -> SyncResult<Arc<Self>> {
        let this;

        if path.exists() {
            this = Self::load(path).await?;
        } else {
            this = Self::default(path);
            this.clone().save().await?;
        }

        Ok(this)
    }
}
impl json::Path for Settings {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSaving for Settings {}
impl json::DirectLoading for Settings {}

