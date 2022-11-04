use actix_web::{get, delete, post, Responder, web};

use crate::{RAW_SCHEDULE, parse, api::Response};
use super::error_response;


#[post("/schedule/daily/convert")]
async fn convert() -> impl Responder {
    let parsing_result = parse::daily(
        RAW_SCHEDULE.ft_daily.clone(), 
        RAW_SCHEDULE.r_weekly.clone()
    ).await;

    if parsing_result.is_err() {
        return error_response(
            parsing_result.unwrap_err()
        ).await;
    }
    let parsed = parsing_result.unwrap();

    Response::from_schedule(parsed).to_json()
}

#[get("/schedule/daily")]
async fn get() -> impl Responder {
    ""
}

#[delete("/schedule/daily")]
async fn delete() -> impl Responder {
    ""
}

#[post("/schedule/weekly/compare")]
async fn compare(bytes: web::Bytes) -> impl Responder {
    ""
}