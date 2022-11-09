use actix_web::{get, delete, post, Responder, web};
use std::sync::Arc;

use crate::data::schedule::Type;

use super::{generic_get, generic_group_get};


#[get("/schedule/weekly")]
async fn get() -> impl Responder {
    let sc_type = Type::Weekly;
    generic_get(sc_type).await
}

#[get("/schedule/weekly?group={group}")]
async fn get_for_group(group: web::Path<String>) -> impl Responder {
    let sc_type = Type::Weekly;
    generic_group_get(sc_type, group).await
}

