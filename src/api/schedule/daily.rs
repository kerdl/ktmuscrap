use actix_web::{get, delete, post, Responder, web};

use crate::{RAW_SCHEDULE, parse};


#[post("/schedule/daily/convert")]
async fn convert() -> impl Responder {
    parse::daily(
        RAW_SCHEDULE.ft_daily.clone(), 
        RAW_SCHEDULE.r_weekly.clone()
    ).await.unwrap();

    ""
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