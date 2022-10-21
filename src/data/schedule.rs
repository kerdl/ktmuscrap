use zip::read::ZipArchive;
use tokio::sync::RwLock;
use std::path::Path;

use crate::DynResult;


pub enum Type {
    FtWeekly,
    FtDaily,
    RWeekly
}
impl Type {
    fn from_string(&self, string: &str) -> Option<Type> {
        match string {
            "ft_weekly" => Some(Type::FtWeekly),
            "ft_daily"  => Some(Type::FtDaily),
            "r_weekly"  => Some(Type::RWeekly),
            _           => None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Type::FtWeekly => "ft_weekly",
            Type::FtDaily  => "ft_daily",
            Type::RWeekly  => "r_weekly"
        }
    }
}


pub struct RawHtml {
    sc_type: Type,
    content: RwLock<String>,
}
impl RawHtml {
    pub fn new(sc_type: Type, content: RwLock<String>) -> RawHtml {
        RawHtml { sc_type, content }
    }

    pub fn from_zip(raw_zip: RawZip) -> RawHtml {
        unimplemented!()

        //let content = raw_zip.extract();

        //RawHtml::new(raw_zip.sc_type, content)
    }
}

pub struct RawZip {
    sc_type: Type,
    content: Option<RwLock<Vec<u8>>>,
}
impl RawZip {
    pub fn new(sc_type: Type, content: Option<RwLock<Vec<u8>>>) -> RawZip {
        RawZip { sc_type, content }
    }

    pub async fn extract(&self) -> DynResult<()> {
        // format directory name as "<schedule_type>_extracted"
        let dir_name = format!("{}_extracted", self.sc_type.to_str());
        // make relative path to this dir
        let dir_path = crate::DATA_PATH.join(dir_name);

        if !dir_path.exists() {
            tokio::fs::create_dir(dir_path).await?;
        }

        Ok(())
    }
}

pub struct Container {
    /// ## `F`ull`t`ime `weekly` schdule ZIP file
    ft_weekly: RwLock<RawZip>,
    /// ## `F`ull`t`ime `daily` schedule ZIP file
    ft_daily: RwLock<RawZip>,
    /// ## `R`emote `weekly` schedule ZIP file
    r_weekly: RwLock<RawZip>,
}
impl Container {
    pub fn new(
        ft_weekly: RwLock<RawZip>, 
        ft_daily: RwLock<RawZip>, 
        r_weekly: RwLock<RawZip>
    ) -> Container {
        Container { ft_weekly, ft_daily, r_weekly }
    }

    pub async fn set_ft_weekly(&self, content: Vec<u8>) {
        let mut field = self.ft_weekly.write().await;
        *field = RawZip::new(Type::FtWeekly, Some(RwLock::new(content)))
    }

    pub async fn set_ft_daily(&self, content: Vec<u8>) {
        let mut field = self.ft_daily.write().await;
        *field = RawZip::new(Type::FtDaily, Some(RwLock::new(content)))
    }

    pub async fn set_r_weekly(&self, content: Vec<u8>) {
        let mut field = self.r_weekly.write().await;
        *field = RawZip::new(Type::RWeekly, Some(RwLock::new(content)))
    }
}
impl Default for Container {
    fn default() -> Container {
        let ft_weekly = RwLock::new(RawZip::new(Type::FtWeekly, None));
        let ft_daily  = RwLock::new(RawZip::new(Type::FtDaily, None));
        let r_weekly  = RwLock::new(RawZip::new(Type::RWeekly, None));

        Container::new(ft_weekly, ft_daily, r_weekly)
    }
}
