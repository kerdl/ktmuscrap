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
async fn generic_loader<'a, Setter, Fut>(
    // The request body
    bytes: web::Bytes,
    // Global instance of raw schedule container
    setter_self: Arc<raw::Container>,
    // Function that will set `bytes` 
    // to schedule storage inside container
    setter: Setter,
    // One of the fields inside `raw::Container`
    storage: Arc<RwLock<raw::Zip>>,
    // Schedule type currently processing
    sc_type: raw::Type
) -> impl Responder
where
    //                 Arc<Self>,        content    awaitable
    Setter: FnOnce(Arc<raw::Container>, web::Bytes) -> Fut,
    // awaitable returns ()
    Fut: Future<Output = ()>
{
    // set recieved ZIP bytes in global schedule container
    setter(setter_self, bytes).await;

    // lock the container field (i.e. `ft_weekly`)
    let sc_storage = storage.read().await;
    // extract ZIP
    let extraction_result = sc_storage.extract().await;

    if extraction_result.is_err() {
        // clean up the mess we just created
        sc_storage.remove_folder().await.unwrap();

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
    let setter_self = crate::RAW_SCHEDULE.clone();
    let setter = raw::Container::set_ft_weekly;
    let storage = crate::RAW_SCHEDULE.ft_weekly.clone();
    let sc_type = raw::Type::FtWeekly;

    generic_loader(bytes, setter_self, setter, storage, sc_type).await
}

#[post("/load/schedule/ft_daily")]
async fn ft_daily(req: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let setter_self = crate::RAW_SCHEDULE.clone();
    let setter = raw::Container::set_ft_daily;
    let storage = crate::RAW_SCHEDULE.ft_daily.clone();
    let sc_type = raw::Type::FtDaily;

    generic_loader(bytes, setter_self, setter, storage, sc_type).await
}

#[post("/load/schedule/r_weekly")]
async fn r_weekly(req: HttpRequest, bytes: web::Bytes) -> impl Responder {
    let setter_self = crate::RAW_SCHEDULE.clone();
    let setter = raw::Container::set_r_weekly;
    let storage = crate::RAW_SCHEDULE.r_weekly.clone();
    let sc_type = raw::Type::RWeekly;

    generic_loader(bytes, setter_self, setter, storage, sc_type).await
}