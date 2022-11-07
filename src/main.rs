pub mod api;
pub mod data;
pub mod parse;
pub mod merge;
pub mod compare;
pub mod fs;
pub mod logger;
pub mod debug;

pub use log::{info, debug};
pub use std::time::Instant;
pub use derive_new;

use actix_web::{web, App, HttpServer};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::{path::PathBuf, sync::Arc};

use logger::Logger;
use data::{schedule, regex, json::LoadOrInit};


static LOGGER: Logger = Logger;
lazy_static! {
    /// ./data
    static ref DATA_PATH: PathBuf = {
        let mut path = PathBuf::new();
        path.push(".");
        path.push("data");
    
        path
    };
    /// ./temp/r_weekly/index.json
    static ref REMOTE_INDEX_PATH: PathBuf = {
        let mut data_path = DATA_PATH.clone();
        data_path.push(data::schedule::raw::Type::RWeekly.to_string());
        data_path.push("index.json");
    
        data_path
    };
    /// ./temp/last_raw.json
    static ref RAW_SCHEDULE_PATH: PathBuf = {
        let mut data_path = DATA_PATH.clone();
        data_path.push("last_raw.json");

        data_path
    };
    /// ./temp/last.json
    static ref LAST_SCHEDULE_PATH: PathBuf = {
        let mut data_path = DATA_PATH.clone();
        data_path.push("last.json");

        data_path
    };

    static ref REGEX: Arc<regex::Container> = {
        Arc::new(regex::Container::default())
    };
}

static RAW_SCHEDULE:  OnceCell<Arc<schedule::raw::Container>> = OnceCell::new();
static LAST_SCHEDULE: OnceCell<Arc<schedule::Last>>           = OnceCell::new();
static REMOTE_INDEX:  OnceCell<Arc<schedule::raw::remote::Index>>  = OnceCell::new();


pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> std::io::Result<()> {

    Logger::init().unwrap();

    if !DATA_PATH.exists() {
        tokio::fs::create_dir(DATA_PATH.as_path()).await?;
        info!("created {:?}", DATA_PATH.as_path());
    }

    let raw_schedule = schedule::raw::Container::load_or_init(
        RAW_SCHEDULE_PATH.to_owned()
    ).await.unwrap();
    RAW_SCHEDULE.set(raw_schedule).unwrap();

    let last_schedule = schedule::Last::load_or_init(
        LAST_SCHEDULE_PATH.to_owned()
    ).await.unwrap();
    LAST_SCHEDULE.set(last_schedule).unwrap();

    let remote_index = schedule::raw::remote::Index::load_or_init(
        REMOTE_INDEX_PATH.to_path_buf()
    ).await.unwrap();
    REMOTE_INDEX.set(remote_index).unwrap();


    /*

    let mut old = parse::remote::html::Parser::from_path(
        data_path.join("r_weekly").join("31.10-06.11.html")
    ).await.unwrap();

    let table = old.table().unwrap();
    let mapping = table.mapping().unwrap();
    let old_page = mapping.page().unwrap();

    let mut new = parse::remote::html::Parser::from_path(
        data_path.join("r_weekly").join("07-12.11.html")
    ).await.unwrap();

    let table = new.table().unwrap();
    let mapping = table.mapping().unwrap();
    let new_page = mapping.page().unwrap();


    let start = Instant::now();
    let compared = compare::schedule::Page::compare(old_page, new_page);

    let dur = start.elapsed();
    info!("comparing took {:?}", dur);

    std::process::exit(0);

    */

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
