use actix_web::{get, delete, post, Responder, web};

use crate::RAW_SCHEDULE;


#[post("/schedule/daily/convert")]
async fn convert() -> impl Responder {
    let parser = crate::parse::fulltime::parse_ft_daily;
    let schedule = RAW_SCHEDULE.ft_daily.clone();

    super::generic_parse(parser, schedule).await
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