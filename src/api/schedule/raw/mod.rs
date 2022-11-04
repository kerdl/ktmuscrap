pub mod ft_weekly;
pub mod ft_daily;
pub mod r_weekly;

use log::info;
use actix_web::{post, delete, Responder, web};
use tokio::sync::RwLock;
use std::sync::Arc;

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
        field_read.delete().await.unwrap();

        return error::ScheduleExtractionFailed::new(
            sc_type,
            extraction_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    Response::ok().to_json()
}

async fn generic_delete(
    // One of the fields inside `raw::Container`
    field: Arc<RwLock<raw::Zip>>,
    // Schedule type currently processing
    sc_type: raw::Type
) -> impl Responder {
    let deletion_result = field.read().await.delete().await;

    if deletion_result.is_err() {
        return error::ScheduleDeletionFailed::new(
            sc_type, 
            deletion_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    Response::ok().to_json()
}


#[delete("/schedule/raw")]
async fn delete() -> impl Responder {
    crate::RAW_SCHEDULE.clone().delete().await;

    Response::ok().to_json()
}