pub mod ft_weekly;
pub mod ft_daily;
pub mod r_weekly;

use actix_web::{delete, Responder, web};
use std::sync::Arc;

use crate::{
    api::{
        error::{self, base::ToApiError}, 
        ToResponse, Response
    }, 
    data::{schedule::{self, raw, Type}, json::SavingLoading}
};


/// ## Generic loader for all schedule types
/// - treat recieved body as a ZIP archive
/// - set ZIP bytes in global schedule container
/// - extract ZIP in file system
async fn generic_load(
    // The request body
    bytes: web::Bytes,
    container: Arc<raw::Container>,
    // One of the fields inside `raw::Container`
    schedule: Arc<raw::Schedule>,
    last_schedule: Arc<schedule::Last>,
    // Schedule type currently processing
    sc_type: raw::Type
) -> impl Responder {

    let zip = schedule.zip.read().await;

    // call content setter
    zip.set_content(bytes).await;

    // extract ZIP
    let extraction_result = zip.extract().await;

    if extraction_result.is_err() {
        // clean up the mess we just created
        zip.delete().await.unwrap();

        return error::ScheduleExtractionFailed::new(
            sc_type,
            extraction_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    schedule.clear_parsed().await;

    if let Err(err) = last_schedule.clear_from_raw_type(&sc_type).await {
        return error::ScheduleClearFailed::new(
            sc_type.clone(),
            match sc_type {
                raw::Type::FtDaily => vec![Type::Daily],
                raw::Type::FtWeekly => vec![Type::Weekly],
                raw::Type::RWeekly => vec![Type::Daily, Type::Weekly]
            },
            err.to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    container.poll_save();

    Response::ok().to_json()
}

async fn generic_delete(
    // One of the fields inside `raw::Container`
    field: Arc<raw::Schedule>,
    // Schedule type currently processing
    sc_type: raw::Type
) -> impl Responder {
    let zip = field.zip.read().await;

    let deletion_result = zip.delete().await;

    if deletion_result.is_err() {
        return error::RawScheduleDeletionFailed::new(
            sc_type, 
            deletion_result.unwrap_err().to_string()
        )
            .to_api_error()
            .to_response()
            .to_json()
    }

    *field.parsed.write().await = None;

    Response::ok().to_json()
}


#[delete("/schedule/raw")]
async fn delete() -> impl Responder {
    crate::RAW_SCHEDULE.get().unwrap().clone().delete().await;

    Response::ok().to_json()
}