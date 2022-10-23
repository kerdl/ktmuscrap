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


pub async fn generic_parser<Parser, ParserFut>(
    parse: Parser
) -> impl Responder 
where
    Parser: FnOnce() -> ParserFut,
    ParserFut: Future<Output = Result<schedule::Page, ApiError>>
{
    let parsing_result = parse().await;

    if parsing_result.is_err() {
        let err = parsing_result.unwrap_err().to_response();
        return web::Json(err)
    }

    let schedule = parsing_result.unwrap();

    web::Json(Response::from_schedule(schedule))
}

#[post("/convert/weekly")]
pub async fn weekly() -> impl Responder {
    let parse = parse::weekly;
    generic_parser(parse).await
}

#[post("/convert/daily")]
pub async fn daily() -> impl Responder {
    let parse = parse::daily;
    generic_parser(parse).await
}