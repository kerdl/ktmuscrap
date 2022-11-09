//! # Generic JSON saving, loading

use std::{
    sync::Arc,
    path::PathBuf
};

use async_trait::async_trait;
use log::warn;
use serde::{
    Serialize,
    de::DeserializeOwned
};

use crate::SyncResult;


pub trait Path {
    fn path(&self) -> PathBuf;
}

#[async_trait]
pub trait ToMiddle<Middle> {
    async fn to_middle(&self) -> Middle;
}

pub trait FromMiddle<Middle> {
    fn from_middle(middle: Arc<Middle>) -> Arc<Self>;
}

#[async_trait]
pub trait DirectSaving
where
    Self: Path + Serialize + Send + Sync + 'static
{
    async fn save(self: Arc<Self>) -> SyncResult<()> {
        let self_ref = self.clone();

        tokio::task::spawn_blocking(move || -> SyncResult<()> {

            let bytes = serde_json::to_vec_pretty(&self_ref)?;
            std::fs::write(self_ref.path(), &bytes)?;

            Ok(())

        }).await.unwrap()
    }

    fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            self.save().await
        });
    }
}

#[async_trait]
pub trait DirectLoading
where
    Self: Path + DeserializeOwned + Send + Sync + 'static
{
    async fn load(path: PathBuf) -> SyncResult<Arc<Self>> {
        tokio::task::spawn_blocking(move || -> SyncResult<Arc<Self>> {

            let string = std::fs::read_to_string(path)?;
            let de = serde_json::from_str(&string)?;

            Ok(Arc::new(de))
        }).await?
    }
}

#[async_trait]
pub trait Saving<Middle>
where
    Self: ToMiddle<Middle> + Send + Sync + 'static,
    Middle: DirectSaving,
{
    async fn save(&self) -> SyncResult<()> {

        let middle = Arc::new(self.to_middle().await);
        middle.save().await?;

        Ok(())
    }
    
    fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            self.save().await
        });
    }
}

#[async_trait]
pub trait Loading<Middle>
where
    Self: FromMiddle<Middle> + Send + Sync + 'static,
    Middle: DirectLoading,
{
    async fn load(path: PathBuf) -> SyncResult<Arc<Self>> {
        let middle = Middle::load(path).await?;
        let primary = Self::from_middle(middle);

        Ok(primary)
    }
}
