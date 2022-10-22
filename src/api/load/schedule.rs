use log::info;
use actix_web::{post, Responder, HttpRequest, web::Bytes};


#[post("/load/schedule/ft_weekly")]
async fn ft_weekly(req: HttpRequest, bytes: Bytes) -> impl Responder {
    crate::RAW_SCHEDULE.set_ft_weekly(bytes).await;
    let extraction_result = crate::RAW_SCHEDULE.ft_weekly
        .read().await
        .extract().await;
    
    if extraction_result.is_err() {

    }

    ""
}

#[post("/load/schedule/ft_daily")]
async fn ft_daily(req: HttpRequest, bytes: Bytes) -> impl Responder {
    crate::RAW_SCHEDULE.set_ft_daily(bytes).await;
    crate::RAW_SCHEDULE.ft_daily.read().await.extract().await;
    ""
}

#[post("/load/schedule/r_weekly")]
async fn r_weekly(req: HttpRequest, bytes: Bytes) -> impl Responder {
    crate::RAW_SCHEDULE.set_r_weekly(bytes).await;
    crate::RAW_SCHEDULE.r_weekly.read().await.extract().await;
    ""
}