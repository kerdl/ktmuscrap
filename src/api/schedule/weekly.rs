use actix_web::{get, delete, post, Responder, web};
use crate::{RAW_SCHEDULE, parse};


#[post("/schedule/weekly/convert")]
async fn convert() -> impl Responder {
    parse::weekly(
        RAW_SCHEDULE.ft_weekly.clone(), 
        RAW_SCHEDULE.r_weekly.clone()
    ).await.unwrap();

    ""
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