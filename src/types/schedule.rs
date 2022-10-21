use tokio::sync::RwLock;

pub struct RawHtml {
    content: RwLock<String>,
}

pub struct RawZip {
    content: RwLock<Vec<u8>>,
}
impl RawZip {
    
}

pub struct _RawZip {
    /// ## `F`ull`t`ime `weekly` schdule ZIP file
    ft_weekly: RwLock<Vec<u8>>,
    /// ## `F`ull`t`ime `weekly` schedule ZIP file
    ft_daily: RwLock<Vec<u8>>,
    /// ## `R`emote `weekly` schedule ZIP file
    r_weekly: RwLock<Vec<u8>>,
}
impl _RawZip {
    pub async fn set_ft_weekly(&self, contents: Vec<u8>) {
        let mut lock = self.ft_weekly.write().await;
        *lock = contents;
    }

    pub async fn set_ft_daily(&self, contents: Vec<u8>) {
        let mut lock = self.ft_daily.write().await;
        *lock = contents;
    }

    pub async fn set_r_weekly(&self, contents: Vec<u8>) {
        let mut lock = self.r_weekly.write().await;
        *lock = contents;
    }
}

