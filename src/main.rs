pub mod api;
pub mod data;
pub mod parse;
pub mod merge;
pub mod fs;
pub mod logger;
pub mod debug;

pub use log::info;
pub use std::time::Instant;
pub use derive_new;
use actix_web::{web, App, HttpServer};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use std::{path::PathBuf, sync::Arc};

use logger::Logger;
use data::{schedule::{self, debug::Dummy}, regex};


static LOGGER: Logger = Logger;
lazy_static! {
    /// ./temp
    static ref TEMP_PATH: PathBuf = {
        let mut path = PathBuf::new();
        path.push(".");
        path.push("temp");
    
        path
    };
    /// ./temp/r_weekly/index.json
    static ref REMOTE_SCHEDULE_INDEX_PATH: PathBuf = {
        let mut temp_path = TEMP_PATH.clone();
        temp_path.push(data::schedule::raw::Type::RWeekly.to_string());
        temp_path.push("index.json");
    
        temp_path
    };

    static ref RAW_SCHEDULE: Arc<schedule::raw::Container> = {
        Arc::new(schedule::raw::Container::default())
    };
    static ref REMOTE_SCHEDULE_INDEX: Arc<RwLock<schedule::raw::Index>> = {
        let sc = schedule::raw::Index::load_or_init(
            REMOTE_SCHEDULE_INDEX_PATH.to_path_buf()
        ).unwrap();

        Arc::new(RwLock::new(sc))
    };
    static ref REGEX: Arc<regex::Container> = {
        Arc::new(regex::Container::default())
    };
    static ref LAST_SCHEDULE: Arc<schedule::Last> = {
        Arc::new(schedule::Last::default())
    };
}

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> std::io::Result<()> {

    Logger::init().unwrap();

    if !TEMP_PATH.exists() {
        tokio::fs::create_dir(TEMP_PATH.as_path()).await?;
        info!("created {:?}", TEMP_PATH.as_path());
    }

    let ft_weekly = data::schedule::Page::dummy();

    let r_weekly = data::schedule::Page::dummy();

    merge::weekly::merge(ft_weekly, r_weekly).await;

    std::process::exit(0);

    // start http server
    HttpServer::new(|| {
        App::new()
            .service(api::schedule::raw::ft_weekly::load)
            .service(api::schedule::raw::ft_weekly::delete)
            .service(api::schedule::raw::ft_daily::load)
            .service(api::schedule::raw::ft_daily::delete)
            .service(api::schedule::raw::r_weekly::load)
            .service(api::schedule::raw::r_weekly::delete)
            .service(api::schedule::raw::delete)

            .service(api::schedule::weekly::convert)
            .service(api::schedule::weekly::get)
            .service(api::schedule::weekly::delete)
            .service(api::schedule::weekly::compare)
            .service(api::schedule::daily::convert)
            .service(api::schedule::daily::get)
            .service(api::schedule::daily::delete)
            .service(api::schedule::daily::compare)

            .app_data(web::PayloadConfig::new(100 * 1024 * 1024)) // 100 mB
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
