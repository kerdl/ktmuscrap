use log::info;
use actix_web::{post, Responder, HttpRequest, web};
use tokio::sync::RwLock;
use std::{future::Future, sync::Arc};

use crate::{
    api::{
        error::{self, base::ToApiError}, 
        ToResponse, Response
    }, 
    data::schedule::raw
};


/// ## Generic loader for all schedule types
/// - treat recieved body as a ZIP archive
/// - set ZIP bytes in global schedule container
/// - extract ZIP in file system
async fn generic_load(
    // The request body
    bytes: web::Bytes,
    // One of the fields inside `raw::Container`
    field: Arc<RwLock<raw::Zip>>,
    // Schedule type currently processing
    sc_type: raw::Type
) -> impl Responder {

    // read lock for this field
    let field_read = field.read().await;
    // call content setter
    field_read.set_content(bytes).await;

    // extract ZIP
    let extraction_result = field_read.extract().await;

    if extraction_result.is_err() {
        // clean up the mess we just created
        field_read.remove_folder().await.unwrap();
        field_read.del_content().await;

        // construct api error
        let err = error::ScheduleExtractionFailed::new(
            sc_type, 
            extraction_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response();
        
        return web::Json(err);
    }

    web::Json(Response::ok())
}


#[post("/load/schedule/ft_weekly")]
async fn ft_weekly(req: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let field = crate::RAW_SCHEDULE.ft_weekly.clone();
    let sc_type = raw::Type::FtWeekly;

    generic_load(bytes, field, sc_type).await
}

#[post("/load/schedule/ft_daily")]
async fn ft_daily(req: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let field = crate::RAW_SCHEDULE.ft_daily.clone();
    let sc_type = raw::Type::FtDaily;

    generic_load(bytes, field, sc_type).await
}

#[post("/load/schedule/r_weekly")]
async fn r_weekly(req: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let field = crate::RAW_SCHEDULE.r_weekly.clone();
    let sc_type = raw::Type::RWeekly;

    generic_load(bytes, field, sc_type).await
}