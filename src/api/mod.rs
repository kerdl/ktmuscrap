pub mod load;
pub mod error;

use serde_derive::Serialize;

use error::base::ApiError;


#[derive(Serialize)]
pub struct Data {

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
}

pub trait ToResponse {
    fn to_response(self) -> Response;
}