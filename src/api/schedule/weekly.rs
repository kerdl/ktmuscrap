use actix_web::{get, delete, post, Responder, web};

use crate::{RAW_SCHEDULE, LAST_SCHEDULE, parse, api::Response};
use super::error_response;


#[post("/schedule/weekly/convert")]
async fn convert() -> impl Responder {
    let parsing_result = parse::weekly(
        RAW_SCHEDULE.ft_weekly.clone(), 
        RAW_SCHEDULE.r_weekly.clone()
    ).await;

    if parsing_result.is_err() {
        return error_response(
            parsing_result.unwrap_err()
        ).await;
    }
    let parsed = LAST_SCHEDULE.weekly.read().await;
    let page = parsed.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}

#[get("/schedule/weekly")]
async fn get() -> impl Responder {
    ""
}

#[delete("/schedule/weekly")]
async fn delete() -> impl Responder {
    ""
}

#[post("/schedule/weekly/compare")]
async fn compare(bytes: web::Bytes) -> impl Responder {
    ""
}