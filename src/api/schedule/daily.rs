use actix_web::{get, delete, post, Responder, web};

use crate::{parse, RAW_SCHEDULE};


#[post("/schedule/daily/convert")]
async fn convert() -> impl Responder {
    parse::remote::parse(RAW_SCHEDULE.r_weekly.clone()).await;

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