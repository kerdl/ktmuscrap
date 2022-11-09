use actix_web::web::Bytes;
use chrono::{NaiveDateTime, Utc, Duration};
use serde::Serialize;
use tokio::{sync::{RwLock, mpsc}, task::JoinHandle};
use std::{time::Instant, sync::Arc, hash::Hash};

use crate::string;


#[derive(Debug)]
pub enum Lifetime {
    Kept,
    Drop
}

#[derive(Debug, Serialize)]
pub struct Interactor {
    #[serde(skip)]
    lifetime_tx: Option<mpsc::Sender<Lifetime>>,
    #[serde(skip)]
    pub ping_rx: Option<Arc<RwLock<mpsc::Receiver<Bytes>>>>,

    pub key: String,
    #[serde(skip)]
    is_connected: Arc<RwLock<bool>>,
}
impl PartialEq for Interactor {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Hash for Interactor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}
impl Eq for Interactor {}
impl Interactor {
    pub fn new(
        lifetime_tx: mpsc::Sender<Lifetime>,
        ping_rx: mpsc::Receiver<Bytes>,
    ) -> Arc<Interactor> {

        let this = Interactor {
            lifetime_tx: Some(lifetime_tx),
            ping_rx: Some(Arc::new(RwLock::new(ping_rx))),
            key: string::random(16),
            is_connected: Arc::new(RwLock::new(false))
        };

        Arc::new(this)
    }

    pub fn from_key(key: String) -> Arc<Interactor> {
        let this = Interactor {
            lifetime_tx: None,
            ping_rx: None,
            key,
            is_connected: Arc::new(RwLock::new(false))
        };

        Arc::new(this)
    }

    pub async fn keep_alive(&self) {
        self.lifetime_tx.as_ref().unwrap()
            .send(Lifetime::Kept).await.unwrap();
    }

    pub async fn wish_drop(&self) {
        self.lifetime_tx.as_ref().unwrap()
            .send(Lifetime::Drop).await.unwrap();
    }

    pub async fn connected(&self) {
        *self.is_connected.write().await = true;
    }

    pub async fn is_connected(&self) -> bool {
        self.is_connected.read().await.clone()
    }
}