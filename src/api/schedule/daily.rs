use actix_web::{get, delete, post, Responder, web, HttpRequest};
use actix_web_actors::ws;
use log::info;

use crate::data::schedule::Type;

use super::{generic_get, generic_group_get, ScheduleGetQuery};


#[get("/schedule/daily")]
async fn get(query: web::Query<ScheduleGetQuery>) -> impl Responder {
    let sc_type = Type::Daily;

    if query.group.is_some() {
        return generic_group_get(
            sc_type,
            query.group.as_ref().unwrap().clone()
        ).await
    }

    generic_get(sc_type).await
}
