use actix_web::{post, delete, Responder, web};

use crate::{data::schedule::raw, api::Response};


#[post("/schedule/raw/ft_daily")]
async fn load(bytes: web::Bytes) -> impl Responder {
    let field = crate::RAW_SCHEDULE.ft_daily.zip.clone();
    let sc_type = raw::Type::FtDaily;

    super::generic_load(bytes, field, sc_type).await
}

#[delete("/schedule/raw/ft_daily")]
async fn delete() -> impl Responder {
    let field = crate::RAW_SCHEDULE.ft_daily.zip.clone();
    let sc_type = raw::Type::FtDaily;

    super::generic_delete(field, sc_type).await
}