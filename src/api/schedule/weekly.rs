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
    data::{schedule::Type, json::SavingLoading}
};
use super::{
    generic_compare,
    error_response
};


#[post("/schedule/weekly/convert")]
async fn convert() -> impl Responder {
    let parsing_result = parse::weekly(
        RAW_SCHEDULE.get().unwrap().ft_weekly.clone(), 
        RAW_SCHEDULE.get().unwrap().r_weekly.clone()
    ).await;

    if let Err(err) = parsing_result {
        return error_response(err).await;
    }

    RAW_SCHEDULE.get().unwrap().clone().poll_save();

    let parsed = LAST_SCHEDULE.get().unwrap().weekly.read().await;
    let page = parsed.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}

#[get("/schedule/weekly")]
async fn get() -> impl Responder {
    let weekly = LAST_SCHEDULE.get().unwrap().weekly.read().await;
    let page = weekly.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}

#[delete("/schedule/weekly")]
async fn delete() -> impl Responder {
    if let Err(err) = LAST_SCHEDULE.get().unwrap().clear_weekly().await {
        return error::ScheduleDeletionFailed::new(
            Type::Weekly,
            err.to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    Response::ok().to_json()
}

#[post("/schedule/weekly/compare")]
async fn compare(bytes: web::Bytes) -> impl Responder {
    generic_compare(Type::Weekly, bytes).await
}