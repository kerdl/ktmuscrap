use actix_web::{post, Responder, web};
use regex::{Regex, bytes};
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::data::regex::Type;
use crate::api::{
    Response, 
    error, 
    ToResponse, 
    error::base::ToApiError
};


/// ## Generic loader for all regexes
/// - validate that recieved body is
/// a valid UTF-8
/// - try to compile the regex
/// - set this regex in a global container
pub async fn genric_loader(
    bytes: web::Bytes,
    field: Arc<RwLock<Option<Regex>>>,
    regex_type: Type
) -> impl Responder {

    // get write lock for this field
    let mut field_write = field.write().await;

    // try to convert HTTP request 
    // body bytes to String
    let regex_str_result = String::from_utf8(bytes.to_vec());

    if regex_str_result.is_err() {
        let err = error::NotAValidUtf8::new(
            regex_str_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response();
        
        return web::Json(err)
    }

    let regex_str = regex_str_result.unwrap();


    // try to compile this regex from string
    let regex_result = Regex::new(&regex_str);

    if regex_result.is_err() {
        let err = error::RegexCompileFailed::new(
            regex_type, 
            regex_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response();
        
        return web::Json(err)
    }

    let regex = regex_result.unwrap();

    *field_write = Some(regex);


    web::Json(Response::ok())
}


#[post("/regex/group")]
async fn group(bytes: web::Bytes) -> impl Responder {
    let field = crate::REGEX.group.clone();
    let regex_type = Type::Group;

    genric_loader(bytes, field, regex_type).await
}

#[post("/regex/date")]
async fn date(bytes: web::Bytes) -> impl Responder {
    let field = crate::REGEX.date.clone();
    let regex_type = Type::Date;

    genric_loader(bytes, field, regex_type).await
}

#[post("/regex/time")]
async fn time(bytes: web::Bytes) -> impl Responder {
    let field = crate::REGEX.time.clone();
    let regex_type = Type::Time;

    genric_loader(bytes, field, regex_type).await
}

#[post("/regex/teacher")]
async fn teacher(bytes: web::Bytes) -> impl Responder {
    let field = crate::REGEX.teacher.clone();
    let regex_type = Type::Teacher;

    genric_loader(bytes, field, regex_type).await
}

#[post("/regex/cabinet")]
async fn cabinet(bytes: web::Bytes) -> impl Responder {
    let field = crate::REGEX.cabinet.clone();
    let regex_type = Type::Cabinet;

    genric_loader(bytes, field, regex_type).await
}