use serde_derive::Serialize;

use crate::api::{Response, ToResponse};
use super::ErrorNum;


#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// ## Indicates user's failure
    /// - i.e. some parameters were not loaded
    UserFailure,
    /// ## Indicates that this server is fucked up
    /// - i.e. reading from disk failed
    InternalFailure,
    /// ## Indicates 3rd party failure
    /// - i.e. schedule is formatted incorrectly
    ParsingFailure
}

#[derive(Serialize, Clone, Debug)]
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