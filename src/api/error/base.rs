use derive_new::new;
use serde_derive::Serialize;
use actix_web::http::StatusCode;

use crate::api::{Response, ToResponse};
use super::ErrorNum;


#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// ## Indicates user's failure
    /// - e.g. some parameters were not loaded
    UserFailure,
    /// ## Indicates that this server had fucked up
    /// - e.g. reading from disk failed
    InternalFailure,
    /// ## Indicates 3rd party failure
    /// - e.g. schedule is formatted incorrectly
    DataFailure
}
impl Kind {
    pub fn status(&self) -> StatusCode {
        match self {
            Kind::UserFailure => StatusCode::BAD_REQUEST,
            Kind::InternalFailure => StatusCode::INTERNAL_SERVER_ERROR,
            Kind::DataFailure => StatusCode::NOT_IMPLEMENTED
        }
    }
}

#[derive(new, Serialize, Clone, Debug)]
pub struct ApiError {
    pub kind: Kind,
    pub error: ErrorNum,
    pub text: String
}
impl ToResponse for ApiError {
    fn to_response(self) -> Response {
        let is_ok = false;
        let data = None;
        let error = Some(self);

        Response {
            is_ok,
            data,
            error
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
impl std::error::Error for ApiError {}

pub trait ToApiError {
    fn to_api_error(&self) -> ApiError;
}