pub mod api;
pub mod types;

use actix_web::{get, web, App, HttpServer, Responder};
use std::io::Result;


#[actix_web::main]
async fn main() -> Result<()> {
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
