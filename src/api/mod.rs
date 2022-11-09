pub mod error;
pub mod schedule;

use derive_new::new;
use actix_web::{web, http::StatusCode};
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

    pub fn to_json(self) -> web::Json<Self> {
        let mut status = StatusCode::OK;

        self.error.as_ref().map(|err| 
            status = match err.kind {
                Kind::UserFailure =>     StatusCode::BAD_REQUEST,
                Kind::InternalFailure => StatusCode::INTERNAL_SERVER_ERROR,
                Kind::DataFailure =>     StatusCode::NOT_IMPLEMENTED
            }
        );

        web::Json(self)
    }
}

pub trait ToResponse {
    fn to_response(self) -> Response;
}