use std::{sync::Arc, path::PathBuf};

use async_trait::async_trait;
use log::warn;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

use crate::SyncResult;


pub trait Path {
    fn path(&self) -> PathBuf;
}

pub trait ToMiddle<Middle>
where
    Middle: DirectSavingLoading
{
    fn to_middle(&self) -> Middle;
}

pub trait FromMiddle<Middle> {
    fn from_middle(middle: Arc<Middle>) -> Arc<Self>;
}

#[async_trait]
pub trait DirectSavingLoading
where 
    Self: Path + Serialize + DeserializeOwned + Send + Sync + 'static
{
    async fn save(self: Arc<Self>) -> SyncResult<()> {
        let self_ref = self.clone();

        if let Err(error) = tokio::task::spawn_blocking(move || -> SyncResult<()> {

            let bytes = serde_json::to_vec_pretty(&self_ref)?;
            std::fs::write(self_ref.path(), &bytes)?;

            Ok(())

        }).await? {
            warn!("error saving to {:?}: {:?}", self.path(), error)
        }

        Ok(())
    }

    async fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            self.save().await
        });
    }

    async fn load(path: PathBuf) -> SyncResult<Arc<Self>> {
        tokio::task::spawn_blocking(move || -> SyncResult<Arc<Self>> {

            let string = std::fs::read_to_string(path)?;
            let de = serde_json::from_str(&string)?;

            Ok(Arc::new(de))
        }).await?
    }
}

#[async_trait]
pub trait SavingLoading<Middle>
where
    Self: ToMiddle<Middle> + FromMiddle<Middle> + Send + Sync + 'static, 
    Middle: DirectSavingLoading
{
    async fn save(&self) -> SyncResult<()> {

        let middle = Arc::new(self.to_middle());
        middle.save().await?;

        Ok(())
    }

    async fn poll_save(self: Arc<Self>) {
        tokio::spawn(async move {
            self.save().await
        });
    }

    async fn load(path: PathBuf) -> SyncResult<Arc<Self>> {

        let middle = Middle::load(path).await?;
        let primary = Self::from_middle(middle);

        Ok(primary)
    }
}