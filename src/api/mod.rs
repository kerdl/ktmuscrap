pub mod error;
pub mod schedule;

use derive_new::new;
use actix_web::{web, http::StatusCode, HttpResponse, HttpResponseBuilder};
use serde_derive::Serialize;
use std::sync::Arc;

use crate::{data::schedule::{self as sc, Interactor}, compare::schedule as cmp};
use error::base::{ApiError, Kind};


#[derive(new, Serialize)]
pub struct Data {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Arc<sc::Page>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactor: Option<Arc<Interactor>>
}
impl Data {
    pub fn from_page(schedule: Arc<sc::Page>) -> Data {
        Data::new(Some(schedule), None)
    }

    pub fn from_interactor(interactor: Arc<Interactor>) -> Data {
        Data::new(None, Some(interactor))
    }
}

#[derive(new, Serialize)]
pub struct Response {
    is_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Data>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ApiError>,
}
impl Response {
    pub fn ok() -> Response {
        Response::new(true, None, None)
    }

    pub fn from_page(schedule: Arc<sc::Page>) -> Response {
        let data = Data::from_page(schedule);
        Response::new(true, Some(data), None)
    }

    pub fn from_interactor(interactor: Arc<Interactor>) -> Response {
        let data = Data::new(None, Some(interactor));
        Response::new(true, Some(data), None)
    }

    pub fn to_json(self) -> HttpResponse {
        let resp = HttpResponseBuilder::new(
            if self.error.is_none() {
                StatusCode::OK
            } else {
                self.error.as_ref().unwrap().kind.status()
            }
        )
            .append_header(("Content-Type", "application/json"))
            .body(serde_json::to_string_pretty(&self).unwrap());

        resp
    }
}

pub trait ToResponse {
    fn to_response(self) -> Response;
}
