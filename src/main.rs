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
use std::{path::PathBuf, sync::Arc, str::FromStr};

use logger::Logger;
use data::{schedule, regex, json::{LoadOrInit, DefaultFromPath, SavingLoading}};


lazy_static! {
    static ref REGEX: Arc<regex::Container> = {
        Arc::new(regex::Container::default())
    };
}

static LOGGER: Logger = Logger;
static DATA: OnceCell<data::Container> = OnceCell::new();


pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> std::io::Result<()> {

    Logger::init().unwrap();


    let data = data::Container::default_from_dir(
        [".", "data"].iter().collect()
    ).await.unwrap();
    DATA.set(data).unwrap();

    DATA.get().unwrap().schedule.index.clone().update_forever().await;

    std::process::exit(0);
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
            .service(api::schedule::weekly::get)
            .service(api::schedule::daily::get)

            .app_data(web::PayloadConfig::new(100 * 1024 * 1024)) // 100 mB
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
