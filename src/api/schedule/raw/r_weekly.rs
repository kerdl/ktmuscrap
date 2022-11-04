use actix_web::{post, delete, Responder, web};

use crate::data::schedule::raw;


#[post("/schedule/raw/r_weekly")]
async fn load(bytes: web::Bytes) -> impl Responder {
    let field = crate::RAW_SCHEDULE.r_weekly.zip.clone();
    let sc_type = raw::Type::RWeekly;

    super::generic_load(bytes, field, sc_type).await
}

#[delete("/schedule/raw/r_weekly")]
async fn delete() -> impl Responder {
    let field = crate::RAW_SCHEDULE.r_weekly.zip.clone();
    let sc_type = raw::Type::RWeekly;

    super::generic_delete(field, sc_type).await
}