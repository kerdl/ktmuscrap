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
    data::{schedule::Type, json::SavingLoading}
};


#[get("/schedule/weekly")]
async fn get() -> impl Responder {
    let weekly = LAST.get().unwrap().weekly.read().await;
    let page = weekly.as_ref().unwrap().clone();

    Response::from_schedule(page).to_json()
}
