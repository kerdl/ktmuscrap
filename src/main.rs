pub mod api;
pub mod data;
pub mod logger;

use log::{info};
use actix_web::{get, web, App, HttpServer, Responder};
use lazy_static::lazy_static;
use std::{path::PathBuf};

use logger::Logger;
use data::schedule;


static LOGGER: Logger = Logger;
lazy_static! {
    /// ./data
    static ref DATA_PATH: PathBuf = {
        let mut path = PathBuf::new();
        path.push(".");
        path.push("data");
    
        path
    };
    static ref SCHEDULE: schedule::Container = {
        schedule::Container::default()
    };
}

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    Logger::init().unwrap();

    if !DATA_PATH.exists() {
        tokio::fs::create_dir(DATA_PATH.as_path()).await?;
        info!("created {:?}", DATA_PATH.as_path());
    }

    // start http server
    HttpServer::new(|| {
        App::new()
            .service(api::load::schedule::ft_weekly)
            .service(api::load::schedule::ft_daily)
            .service(api::load::schedule::r_weekly)
            .service(api::load::regex::group)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
