use actix_web::{post, delete, Responder, web};

use crate::{data::schedule::raw, api::Response};


#[post("/schedule/raw/ft_daily")]
async fn load(bytes: web::Bytes) -> impl Responder {
    let container = crate::RAW_SCHEDULE.get().unwrap().clone();
    let schedule = crate::RAW_SCHEDULE.get().unwrap().ft_daily.clone();
    let last_schedule = crate::LAST_SCHEDULE.get().unwrap().clone();
    let sc_type = raw::Type::FtDaily;

    super::generic_load(
        bytes,
        container,
        schedule,
        last_schedule,
        sc_type
    ).await
}

#[delete("/schedule/raw/ft_daily")]
async fn delete() -> impl Responder {
    let field = crate::RAW_SCHEDULE.get().unwrap().ft_daily.clone();
    let sc_type = raw::Type::FtDaily;

    super::generic_delete(field, sc_type).await
}