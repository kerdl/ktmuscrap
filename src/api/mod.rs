pub mod error;
pub mod schedule;

use serde_derive::Serialize;

use crate::data::schedule as sc;
use error::base::ApiError;


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
}

pub trait ToResponse {
    fn to_response(self) -> Response;
}