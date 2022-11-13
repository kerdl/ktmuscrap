pub mod error;
pub mod schedule;

use chrono::{Duration, NaiveDateTime};
use derive_new::new;
use actix_web::{web, http::StatusCode, HttpResponse, HttpResponseBuilder};
use serde_derive::Serialize;
use std::sync::Arc;

use crate::{data::schedule::{self as sc, Interactor, Notify}, compare::schedule as cmp};
use error::base::{ApiError, Kind};


#[derive(new, Serialize)]
pub struct Data {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Arc<sc::Page>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactor: Option<Arc<Interactor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<Arc<Notify>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<std::time::Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_update: Option<NaiveDateTime>
}
impl Data {
    pub fn from_page(schedule: Arc<sc::Page>) -> Data {
        Data::new(Some(schedule), None, None, None, None, None)
    }

    pub fn from_interactor(interactor: Arc<Interactor>) -> Data {
        Data::new(None, Some(interactor), None, None, None, None)
    }

    pub fn from_notify(notify: Arc<Notify>) -> Data {
        Data::new(None, None, Some(notify), None, None, None)
    }

    pub fn from_url(url: String) -> Data {
        Data::new(None, None, None, Some(url), None, None)
    }

    pub fn from_period(period: std::time::Duration) -> Data {
        Data::new(None, None, None, None, Some(period), None)
    }

    pub fn from_last_update(last_update: NaiveDateTime) -> Data {
        Data::new(None, None, None, None, None, Some(last_update))
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
        let data = Data::from_interactor(interactor);
        Response::new(true, Some(data), None)
    }

    pub fn from_notify(notify: Arc<Notify>) -> Response {
        let data = Data::from_notify(notify);
        Response::new(true, Some(data), None)
    }

    pub fn from_url(url: String) -> Response {
        let data = Data::from_url(url);
        Response::new(true, Some(data), None)
    }

    pub fn from_period(period: std::time::Duration) -> Response {
        let data = Data::from_period(period);
        Response::new(true, Some(data), None)
    }

    pub fn from_last_update(last_update: NaiveDateTime) -> Response {
        let data = Data::from_last_update(last_update);
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
