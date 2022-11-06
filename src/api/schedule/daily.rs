use log::warn;
use actix_web::{get, delete, post, Responder, web};

use crate::{
    RAW_SCHEDULE,
    LAST_SCHEDULE,
    parse,
    api::{
        Response,
        error::{
            self,
            base::ToApiError
        },
        ToResponse
    },
    data::schedule::Type,
};
use super::{
    generic_compare,
    error_response
};


#[post("/schedule/daily/convert")]
async fn convert() -> impl Responder {
    let parsing_result = parse::daily(
        RAW_SCHEDULE.get().unwrap().ft_daily.clone(), 
        RAW_SCHEDULE.get().unwrap().r_weekly.clone()
    ).await;

    if parsing_result.is_err() {
        return error_response(
            parsing_result.unwrap_err()
        ).await;
    }

    RAW_SCHEDULE.get().unwrap().save().await;

    let parsed = LAST_SCHEDULE.get().unwrap().daily.read().await;
    let page = parsed.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}

#[get("/schedule/daily")]
async fn get() -> impl Responder {
    let daily = LAST_SCHEDULE.get().unwrap().daily.read().await;
    let page = daily.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}

#[delete("/schedule/daily")]
async fn delete() -> impl Responder {
    if let Err(err) = LAST_SCHEDULE.get().unwrap().clear_daily().await {
        return error::ScheduleDeletionFailed::new(
            Type::Daily,
            err.to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    Response::ok().to_json()
}

#[post("/schedule/daily/compare")]
async fn compare(bytes: web::Bytes) -> impl Responder {
    generic_compare(Type::Daily, bytes).await
}