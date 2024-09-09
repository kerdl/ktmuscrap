pub mod api;
pub mod data;
pub mod parse;
pub mod merge;
pub mod compare;
pub mod fs;
pub mod string;
pub mod logger;
pub mod debug;

pub use log::{info, error, debug};
pub use std::time::Instant;
pub use derive_new;

use actix_web::{App, HttpServer};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::{sync::Arc, net::SocketAddr};

use logger::Logger;
use data::regex;


lazy_static! {
    static ref REGEX: Arc<regex::Container> = {
        Arc::new(regex::Container::default())
    };
}

static LOGGER: Logger = Logger;
static DATA: OnceCell<data::Container> = OnceCell::new();


pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    Logger::init().unwrap();

    let data = data::Container::default_from_dir(
        [".", "data"].iter().collect()
    ).await.unwrap();
    DATA.set(data).unwrap();

    if DATA.get().unwrap().schedule.index.types.len() < 1 {
        let example = crate::data::schedule::raw::index::MiddleSchedule::example();
        let json_example = serde_json::to_string_pretty(&example).unwrap();

        error!(
            "before running, see ./data/schedule/index.json \
            and fill in the schedule types manually"
        );
        info!(
            "example:\n {}", json_example
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "no schedule types"
        ));
    }

    DATA.get().unwrap().schedule.index.clone().update_forever().await;

    let addr = DATA.get().unwrap().settings.server.address.clone();
    info!("http server will be ran on {}", addr);
    // start http server
    HttpServer::new(|| {
        App::new()
            .service(api::schedule::groups::get)
            .service(api::schedule::teachers::get)
            .service(api::schedule::updates)
            .service(api::schedule::updates_period)
            .service(api::schedule::updates_last)
    })
        .bind(addr)?
        .run()
        .await
}
