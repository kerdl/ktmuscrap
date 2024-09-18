use std::{sync::Arc, path::PathBuf};
use palette::IntoColor;
use serde_derive::{Serialize, Deserialize};
use crate::{SyncResult, data::json::{
    self,
    Saving,
    Loading
}};


#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Parsing {
    pub fulltime_hex: String,
    pub fulltime_lab: palette::Lab,
    pub remote_hex: String,
    pub remote_lab: palette::Lab
}
impl json::ToMiddle<MiddleParsing> for Parsing {
    async fn to_middle(&self) -> MiddleParsing {
        MiddleParsing {
            fulltime_color: self.fulltime_hex.clone(),
            remote_color: self.remote_hex.clone()
        }
    }
}
impl json::FromMiddle<MiddleParsing> for Parsing {
    fn from_middle(middle: Arc<MiddleParsing>) -> Arc<Self> {
        Arc::new(Self::from_hex(
            middle.fulltime_color.clone(),
            middle.remote_color.clone()
        ).unwrap())
    }
}
impl Parsing {
    fn default() -> Self {
        Self::from_hex(
            "#fce5cd".to_string(),
            "#c6d9f0".to_string()
        ).unwrap()
    }

    pub fn from_hex(fulltime: String, remote: String)
        -> Result<Self, palette::rgb::FromHexError>
    {
        let fulltime_rgb8 = fulltime.parse::<palette::Srgb<u8>>()?;
        let remote_rgb8 = remote.parse::<palette::Srgb<u8>>()?;
        let fulltime_rgb32: palette::Srgb = fulltime_rgb8.into();
        let remote_rgb32: palette::Srgb = remote_rgb8.into();
        let fulltime_lab: palette::Lab = fulltime_rgb32.into_color();
        let remote_lab: palette::Lab = remote_rgb32.into_color();

        let this = Self {
            fulltime_hex: fulltime,
            fulltime_lab,
            remote_hex: remote,
            remote_lab,
        };

        Ok(this)
    }
}


#[derive(Serialize, Deserialize, Clone)]
pub struct MiddleParsing {
    pub fulltime_color: String,
    pub remote_color: String
}


#[derive(Debug)]
pub struct Settings {
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
impl json::ToMiddle<MiddleSettings> for Settings {
    async fn to_middle(&self) -> MiddleSettings {
        MiddleSettings {
            path: self.path.clone(),
            server: self.server.clone(),
            parsing: self.parsing.to_middle().await
        }
    }
}
impl json::FromMiddle<MiddleSettings> for Settings {
    fn from_middle(middle: Arc<MiddleSettings>) -> Arc<Self> {
        let this = Self {
            path: middle.path.clone(),
            server: middle.server.clone(),
            // don't ask me why so stupid
            // and silly and retarded
            // and idiot and autism
            parsing: (*Parsing::from_middle(
                Arc::new(middle.parsing.clone())
            )).clone()
        };

        Arc::new(this)
    }
}
impl json::Saving<MiddleSettings> for Settings {}
impl json::Loading<MiddleSettings> for Settings {}


#[derive(Serialize, Deserialize)]
pub struct MiddleSettings {
    #[serde(skip)]
    path: PathBuf,
    pub server: Server,
    pub parsing: MiddleParsing
}
impl json::Path for MiddleSettings {
    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
impl json::DirectSaving for MiddleSettings {}
impl json::DirectLoading for MiddleSettings {}
