pub mod raw;
pub mod weekly;
pub mod daily;

use actix_web::Responder;
use tokio::sync::RwLock;
use std::{future::Future, sync::Arc};

use crate::{
    api::{self, error::base::ToApiError, Response, ToResponse}, 
    data::schedule::raw::Zip,
    SyncResult
};


pub async fn generic_parse<Parser, FutParsed>(
    parser: Parser,
    schedule: Arc<RwLock<Zip>>
) -> impl Responder
where
    Parser: FnOnce(Arc<RwLock<Zip>>) -> FutParsed,
    FutParsed: Future<Output = SyncResult<()>>
{
    let parsed = parser(schedule).await;

    if parsed.is_err() {
        let err = parsed.unwrap_err();

        match err {
            // io error from tokio
            tokio_err if err.is::<tokio::io::Error>() => {

                let err: &tokio::io::Error = {
                    tokio_err.downcast_ref().unwrap()
                };

                return api::error::IoError::new(format!("{}", err))
                    .to_api_error()
                    .to_response()
                    .to_json()
            }
            // anything else, retard?
            _ => {
                return api::error::Unknown::new(err.to_string())
                    .to_api_error()
                    .to_response()
                    .to_json()
            }
        }

    }

    Response::ok().to_json()
}