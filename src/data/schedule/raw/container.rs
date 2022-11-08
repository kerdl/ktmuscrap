use async_trait::async_trait;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{
    sync::Arc,
    path::PathBuf
};

use crate::{
    DynResult,
    data::{
        schedule::Page,
        json::{
            self,
            Path,
        }
    },
};
use super::{
    Zip,
    Type as RawType
};


#[derive(Clone, Debug)]
pub struct Schedule {
    pub zip: Arc<RwLock<Zip>>,
    pub parsed: Arc<RwLock<Option<Arc<Page>>>>
}
impl Schedule {
    pub fn from_sc_type(sc_type: RawType) -> Schedule {
        Schedule {
            zip:    Arc::new(RwLock::new(Zip::from_sc_type(sc_type))),
            parsed: Arc::new(RwLock::new(None))
        }
    }

    pub async fn clear_parsed(&self) {
        *self.parsed.write().await = None;
    }
}
