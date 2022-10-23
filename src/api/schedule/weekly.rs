use actix_web::{get, delete, post, Responder, web};


#[get("/schedule/weekly")]
async fn get() -> impl Responder {
    ""
}

#[delete("/schedule/weekly")]
async fn delete() -> impl Responder {
    ""
}

#[post("/schedule/weekly/compare")]
async fn compare(bytes: web::Bytes) -> impl Responder {
    ""
}