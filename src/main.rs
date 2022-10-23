pub mod api;
pub mod data;
pub mod parse;
pub mod logger;

use log::{info};
use actix_web::{get, web, App, HttpServer, Responder};
use lazy_static::lazy_static;
use std::{path::PathBuf, sync::Arc};

use logger::Logger;
use data::{schedule, regex};


static LOGGER: Logger = Logger;
lazy_static! {
    /// ./temp
    static ref TEMP_PATH: PathBuf = {
        let mut path = PathBuf::new();
        path.push(".");
        path.push("temp");
    
        path
    };
    static ref RAW_SCHEDULE: Arc<schedule::raw::Container> = {
        Arc::new(schedule::raw::Container::default())
    };
    static ref REGEX: Arc<regex::Container> = {
        Arc::new(regex::Container::default())
    };
    static ref LAST_SCHEDULE: Arc<schedule::Last> = {
        Arc::new(schedule::Last::default())
    };
}

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    Logger::init().unwrap();

    if !TEMP_PATH.exists() {
        tokio::fs::create_dir(TEMP_PATH.as_path()).await?;
        info!("created {:?}", TEMP_PATH.as_path());
    }

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

            .service(api::schedule::weekly::get)
            .service(api::schedule::weekly::delete)
            .service(api::schedule::weekly::compare)
            .service(api::schedule::daily::get)
            .service(api::schedule::daily::delete)
            .service(api::schedule::daily::compare)

            .service(api::regex::group)
            .service(api::regex::date)
            .service(api::regex::time)
            .service(api::regex::teacher)
            .service(api::regex::cabinet)

            .app_data(web::PayloadConfig::new(100 * 1024 * 1024)) // 100 mB
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
