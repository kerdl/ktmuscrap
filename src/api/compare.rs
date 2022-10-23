use actix_web::{post, Responder, web};
use regex::{Regex, bytes};
use tokio::sync::RwLock;
use std::future::Future;
use std::sync::Arc;

use crate::parse;
use crate::data::{regex::Type, schedule};
use crate::api::{
    Response, 
    ToResponse, 
    error::base::ApiError
};


#[post("/compare/weekly")]
pub async fn weekly() -> impl Responder {
    ""
}

#[post("/compare/daily")]
pub async fn daily() -> impl Responder {
    ""
}