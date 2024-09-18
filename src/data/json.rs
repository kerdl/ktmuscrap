//! # Generic JSON saving, loading

use std::{
    sync::Arc,
    path::PathBuf
};

use thiserror::Error;
use serde::{
    Serialize,
    de::DeserializeOwned
};

use crate::SyncResult;


#[derive(Error, Debug)]
#[error("saving error")]
pub enum SaveError {
    SerializeError(serde_json::Error),
    WriteError(std::io::Error)
}
impl From<serde_json::Error> for SaveError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerializeError(value)
    }
}
impl From<std::io::Error> for SaveError {
    fn from(value: std::io::Error) -> Self {
        Self::WriteError(value)
    }
}


pub trait Path {
    fn path(&self) -> PathBuf;
}


pub trait ToMiddle<Middle> {
    fn to_middle(&self) -> impl std::future::Future<Output = Middle> + Send;
}

pub trait FromMiddle<Middle> {
    fn from_middle(middle: Arc<Middle>) -> Arc<Self>;
}

pub trait DirectSaving
where
    Self: Path + Serialize + Send + Sync + 'static
{
    fn save(self: Arc<Self>) -> impl std::future::Future<Output = Result<(), SaveError>> + Send {
        async {
            tokio::task::spawn_blocking(move || -> Result<(), SaveError> {
                let bytes = serde_json::to_vec_pretty(&self)?;
                std::fs::write(self.path(), &bytes)?;
                Ok(())
            }).await.unwrap()
        }
    }

    fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            self.save().await
        });
    }
}

pub trait DirectLoading
where
    Self: Path + DeserializeOwned + Send + Sync + 'static
{
    fn load(path: PathBuf) -> impl std::future::Future<Output = SyncResult<Arc<Self>>> + Send {
        async {
            tokio::task::spawn_blocking(move || -> SyncResult<Arc<Self>> {
                let string = std::fs::read_to_string(path)?;
                let de = serde_json::from_str(&string)?;

                Ok(Arc::new(de))
            }).await?
        }
    }
}

pub trait Saving<Middle>
where
    Self: ToMiddle<Middle> + Send + Sync + 'static,
    Middle: DirectSaving,
{
    fn save(&self) -> impl std::future::Future<Output = Result<(), SaveError>> + Send {
        async {
            let middle = Arc::new(self.to_middle().await);
            middle.save().await?;
            Ok(())
        }
    }
    
    fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            self.save().await
        });
    }
}

pub trait Loading<Middle>
where
    Self: FromMiddle<Middle> + Send + Sync + 'static,
    Middle: DirectLoading,
{
    fn load(path: PathBuf) -> impl std::future::Future<Output = SyncResult<Arc<Self>>> + Send {
        async {
            let middle = Middle::load(path).await?;
            let primary = Self::from_middle(middle);

            Ok(primary)
        }
    }
}
