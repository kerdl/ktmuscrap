pub mod api;
pub mod data;
pub mod parse;
pub mod merge;
pub mod compare;
pub mod fs;
pub mod string;
pub mod lifetime;
pub mod logger;

pub use log::{info, error, debug};
pub use std::time::Instant;
pub use derive_new;

use actix_web::{App, HttpServer};
use logger::Logger;
use data::regex;


static LOGGER: Logger = Logger;
static mut REGEX: *const regex::Container = std::ptr::null();
static mut DATA: *const data::Container = std::ptr::null();

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type SyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;


pub fn regexes() -> &'static regex::Container {
    unsafe { &*(REGEX) }
}

pub fn options() -> &'static data::Container {
    unsafe { &*(DATA) }
}


#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    Logger::init().unwrap();

    let data_path = [".", "data"].iter().collect();

    let regex_own = regex::Container::default();
    let data_own = data::Container::default_from_dir(data_path).await.unwrap();
   
    unsafe {
        REGEX = &regex_own;
        DATA = &data_own;
    }

    if options().schedule.index.types.len() < 1 {
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

    options().schedule.index.clone().update_forever().await;

    let addr = options().settings.server.address.clone();
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
