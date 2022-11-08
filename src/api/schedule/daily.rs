use actix_web::{get, delete, post, Responder, web};

use crate::{
    RAW,
    LAST,
    parse,
    api::{
        Response,
        error::{
            self,
            base::ToApiError
        },
        ToResponse
    },
    data::{schedule::Type, json::SavingLoading},
};


#[get("/schedule/daily")]
async fn get() -> impl Responder {
    let daily = LAST.get().unwrap().daily.read().await;
    let page = daily.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}
