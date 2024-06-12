use actix_web::{get, delete, post, Responder, web, HttpRequest};
use actix_web_actors::ws;
use log::info;

use crate::{data::schedule::Type, api::Response, DATA};


#[get("/schedule/raw/tchr_r_weekly/friendly_url")]
async fn friendly_url() -> impl Responder {
    Response::from_url(
        DATA.get().unwrap()
        .schedule.index.tchr_r_weekly().await
        .friendly_url.clone()
    ).to_json()
}