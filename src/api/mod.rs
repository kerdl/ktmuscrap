pub mod error;
pub mod schedule;

use actix_web::{web, http::StatusCode};
use serde_derive::Serialize;

use crate::data::schedule as sc;
use error::base::{ApiError, Kind};


#[derive(Serialize)]
pub struct Data {
    pub schedule: Option<sc::Page>
}
impl Data {
    pub fn new(schedule: Option<sc::Page>) -> Data {
        Data { schedule }
    }

    pub fn from_schedule(schedule: sc::Page) -> Data {
        Data::new(Some(schedule))
    }
}

#[derive(Serialize)]
pub struct Response {
    is_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Data>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ApiError>,
}
impl Response {
    pub fn new(is_ok: bool, data: Option<Data>, error: Option<ApiError>) -> Response {
        Response { is_ok, data, error }
    }
    
    pub fn ok() -> Response {
        Response::new(true, None, None)
    }

    pub fn from_schedule(schedule: sc::Page) -> Response {
        let data = Data::from_schedule(schedule);

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