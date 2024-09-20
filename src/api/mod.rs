pub mod error;
pub mod schedule;

use chrono::NaiveDateTime;
use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder};
use serde_derive::Serialize;
use std::sync::Arc;

use crate::data::schedule as sc;
use error::base::ApiError;


#[derive(Serialize)]
pub struct Updates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<std::time::Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<NaiveDateTime>
}
impl Default for Updates {
    fn default() -> Self {
        Self {
            period: None,
            last: None
        }
    }
}
impl Updates {
    pub fn from_period(period: std::time::Duration) -> Self {
        Self {
            period: Some(period),
            ..Default::default()
        }
    }

    pub fn from_last(last: NaiveDateTime) -> Self {
        Self {
            last: Some(last),
            ..Default::default()
        }
    }
}

#[derive(Serialize)]
pub struct Data {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Arc<sc::Page>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates: Option<Updates>
}
impl Default for Data {
    fn default() -> Self {
        Self {
            page: None,
            updates: None
        }
    }
}
impl Data {
    pub fn from_page(page: Arc<sc::Page>) -> Self {
        Self {
            page: Some(page),
            ..Default::default()
        }
    }

    pub fn from_updates(updates: Updates) -> Self {
        Self {
            updates: Some(updates),
            ..Default::default()
        }
    }

    pub fn from_updates_period(period: std::time::Duration) -> Self {
        Self::from_updates(Updates::from_period(period))
    }

    pub fn from_updates_last(last: NaiveDateTime) -> Self {
        Self::from_updates(Updates::from_last(last))
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
impl Default for Response {
    fn default() -> Self {
        Self {
            is_ok: true,
            data: None,
            error: None
        }
    }
}
impl Response {
    pub fn ok() -> Self {
        Self::default()
    }

    pub fn from_page(schedule: Arc<sc::Page>) -> Self {
        Self {
            data: Some(Data::from_page(schedule)),
            ..Default::default()
        }
    }

    pub fn from_updates(updates: Updates) -> Self {
        Self {
            data: Some(Data::from_updates(updates)),
            ..Default::default()
        }
    }

    pub fn from_updates_period(period: std::time::Duration) -> Self {
        Self {
            data: Some(Data::from_updates(Updates::from_period(period))),
            ..Default::default()
        }
    }

    pub fn from_updates_last(last: NaiveDateTime) -> Self {
        Self {
            data: Some(Data::from_updates(Updates::from_last(last))),
            ..Default::default()
        }
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
