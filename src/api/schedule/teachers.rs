use actix_web::{get, Responder, web};

use crate::data::schedule::raw::Kind;
use crate::api::schedule::{generic_get, ScheduleGetNameQuery};


#[get("/schedule/teachers")]
async fn get(query: web::Query<ScheduleGetNameQuery>) -> impl Responder {
    generic_get(Kind::Teachers, query).await
}
