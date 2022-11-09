use serde::Serialize;
use tokio::sync::RwLock;
use std::{time::Instant, sync::Arc, hash::Hash};

use crate::string;


#[derive(Debug, Clone, Serialize)]
pub struct Interactor {
    key: String,
    #[serde(skip)]
    last_ping: Arc<RwLock<Instant>>
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
    pub fn new() -> Interactor {
        Interactor {
            key: string::random(8),
            last_ping: Arc::new(RwLock::new(Instant::now()))
        }
    }

    pub async fn just_pinged(&mut self) {
        *self.last_ping.write().await = Instant::now()
    }
}