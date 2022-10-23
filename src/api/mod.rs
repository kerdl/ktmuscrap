pub mod error;
pub mod load;
pub mod compare;
pub mod convert;

use serde_derive::Serialize;

use crate::data::schedule;
use error::base::ApiError;


#[derive(Serialize)]
pub struct Data {
    pub schedule: Option<schedule::Page>
}
impl Data {
    pub fn new(schedule: Option<schedule::Page>) -> Data {
        Data { schedule }
    }

    pub fn from_schedule(schedule: schedule::Page) -> Data {
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

    pub fn from_schedule(schedule: schedule::Page) -> Response {
        let data = Data::from_schedule(schedule);

        Response::new(true, Some(data), None)
    }
}

pub trait ToResponse {
    fn to_response(self) -> Response;
}