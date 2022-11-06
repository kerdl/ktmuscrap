pub mod raw;
pub mod weekly;
pub mod daily;

use actix_web::{web, Responder};
use tokio::sync::RwLock;
use std::{future::Future, sync::Arc, error::Error};

use crate::{
    api::{self, error::base::ToApiError, Response, ToResponse}, 
    data::schedule::{raw::Zip, Type, Page},
    SyncResult, LAST_SCHEDULE, compare::{self, DetailedCmp}
};


pub async fn generic_compare(
    sc_type: Type,
    bytes: web::Bytes
) -> impl Responder {
    let page: Result<Page, serde_json::Error> = {
        serde_json::from_slice(&bytes)
    };

    if let Err(error) = page {
        return api::error::PageParsingFailed::new(
            sc_type,
            error.to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    let users_page = page.unwrap();
    let last_page = match sc_type {
        Type::Daily => LAST_SCHEDULE.get().unwrap().daily.read().await,
        Type::Weekly => LAST_SCHEDULE.get().unwrap().weekly.read().await,
    };

    if last_page.is_none() {
        return api::error::NoLastSchedule::new(sc_type)
            .to_api_error()
            .to_response()
            .to_json()
    }

    let last_page = last_page.as_ref().unwrap().clone();

    let comparison = compare::schedule::Page::compare(
        users_page,
        (*last_page).clone()
    );

    Response::from_comparison(comparison.clone())
        .to_json()
}

pub async fn error_response(
    err: Box<dyn Error + Send + Sync>
) -> web::Json<Response> {
    match err {
        // io error from tokio
        tokio_err if err.is::<tokio::io::Error>() => {

            let err: &tokio::io::Error = {
                tokio_err.downcast_ref().unwrap()
            };

            return api::error::IoError::new(format!("{}", err))
                .to_api_error()
                .to_response()
                .to_json()
        }
        // anything else, retard?
        _ => {
            return api::error::Unknown::new(err.to_string())
                .to_api_error()
                .to_response()
                .to_json()
        }
    }
}