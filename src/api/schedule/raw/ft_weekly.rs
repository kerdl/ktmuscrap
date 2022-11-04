use actix_web::{post, delete, Responder, web};

use crate::data::schedule::raw;


#[post("/schedule/raw/ft_weekly")]
async fn load(bytes: web::Bytes) -> impl Responder {
    let field = crate::RAW_SCHEDULE.ft_weekly.zip.clone();
    let sc_type = raw::Type::FtWeekly;

    super::generic_load(bytes, field, sc_type).await
}

#[delete("/schedule/raw/ft_weekly")]
async fn delete() -> impl Responder {
    let field = crate::RAW_SCHEDULE.ft_weekly.zip.clone();
    let sc_type = raw::Type::FtWeekly;

    super::generic_delete(field, sc_type).await
}