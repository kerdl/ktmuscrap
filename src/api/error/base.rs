use serde_derive::Serialize;

use crate::api::{Response, ToResponse};
use super::ErrorNum;


#[derive(Serialize)]
pub enum Kind {
    /// ## Indicates user's failure
    /// - i.e. some parameters were not loaded
    #[serde(rename = "user_failure")]
    UserFailure,
    /// ## Indicates 3rd party failure
    /// - i.e. schedule is formatted incorrectly
    #[serde(rename = "parsing_failure")]
    ParsingFailure
}

#[derive(Serialize)]
pub struct ApiError {
    pub kind: Kind,
    pub error: ErrorNum,
    pub text: String
}
impl ApiError {
    pub const fn new(kind: Kind, error: ErrorNum, text: String) -> ApiError {
        ApiError { kind, error, text }
    }
}
impl ToResponse for ApiError {
    fn to_response(self) -> Response {
        let is_ok = false;
        let data = None;
        let error = Some(self);

        Response::new(is_ok, data, error)
    }
}

pub trait ToApiError {
    fn to_api_error(&self) -> ApiError;
}