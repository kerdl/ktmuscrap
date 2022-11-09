use actix_web::{get, delete, post, Responder, web, HttpRequest};
use actix_web_actors::ws;

use crate::data::schedule::Type;

use super::{generic_get, generic_group_get};


#[get("/schedule/daily")]
async fn get() -> impl Responder {
    let sc_type = Type::Daily;
    generic_get(sc_type).await
}

#[get("/schedule/daily?group={group}")]
async fn get_for_group(group: web::Path<String>) -> impl Responder {
    let sc_type = Type::Daily;
    generic_group_get(sc_type, group).await
}
