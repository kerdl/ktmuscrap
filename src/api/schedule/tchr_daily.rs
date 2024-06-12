use actix_web::{get, delete, post, Responder, web, HttpRequest};
use actix_web_actors::ws;
use log::info;

use crate::data::schedule::Type;

use super::{generic_tchr_get, generic_teacher_get, TchrScheduleGetQuery};


#[get("/schedule/tchr_daily")]
async fn get(query: web::Query<TchrScheduleGetQuery>) -> impl Responder {
    let sc_type = Type::Daily;

    if query.teacher.is_some() {
        return generic_teacher_get(
            sc_type,
            query.teacher.as_ref().unwrap().clone()
        ).await
    }

    generic_tchr_get(sc_type).await
}
