pub mod api;
pub mod data;
pub mod parse;
pub mod merge;
pub mod compare;
pub mod fs;
pub mod string;
pub mod logger;
pub mod debug;

pub use log::{info, debug};
pub use std::time::Instant;
pub use derive_new;

use actix_web::{web, App, HttpServer};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::{path::PathBuf, sync::Arc, str::FromStr, net::SocketAddr, time::Duration};

use logger::Logger;
use data::{schedule, regex};


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

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    Logger::init().unwrap();


    let data = data::Container::default_from_dir(
        [".", "data"].iter().collect()
    ).await.unwrap();
    DATA.set(data).unwrap();


    DATA.get().unwrap().schedule.index.clone().update_forever().await;


    info!("http server will be ran on {}", addr);

    // start http server
    HttpServer::new(|| {
        App::new()
            .service(api::schedule::raw::ft_daily::friendly_url)
            .service(api::schedule::raw::ft_weekly::friendly_url)
            .service(api::schedule::raw::r_weekly::friendly_url)
            .service(api::schedule::interact)
            .service(api::schedule::interact_keep_alive)
            .service(api::schedule::key_is_valid)
            .service(api::schedule::updates)
            .service(api::schedule::update)
            .service(api::schedule::update_period)
            .service(api::schedule::update_last)
            .service(api::schedule::daily::get)
            .service(api::schedule::weekly::get)
    })
        .bind(addr)?
        .run()
        .await
}
