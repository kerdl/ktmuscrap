pub mod weekly;
pub mod daily;

use actix::{Actor, StreamHandler, SpawnHandle, AsyncContext};
use actix_web::{web, Responder, get, post, HttpRequest};
use actix_web_actors::ws;
use log::{info, debug};
use tokio::sync::watch;
use std::sync::Arc;

use crate::{DATA, data::schedule::{self, Type}, string};
use super::{error::{self, base::ToApiError}, ToResponse, Response};


async fn generic_get(sc_type: Type) -> impl Responder {
    let schedule = match sc_type {
        Type::Weekly => DATA.get().unwrap().schedule.last.weekly.read().await,
        Type::Daily  => DATA.get().unwrap().schedule.last.daily.read().await
    };

    if schedule.is_none() {
        return error::NoLastSchedule::new(sc_type)
            .to_api_error()
            .to_response()
            .to_json()
    }

    Response::from_page(
        schedule.as_ref().unwrap().clone()
    ).to_json()
}

async fn generic_group_get(sc_type: Type, group: web::Path<String>) -> impl Responder {
    let schedule = match sc_type {
        Type::Weekly => DATA.get().unwrap().schedule.last.weekly.read().await,
        Type::Daily  => DATA.get().unwrap().schedule.last.daily.read().await
    };

    if schedule.is_none() {
        return error::NoLastSchedule::new(sc_type)
            .to_api_error()
            .to_response()
            .to_json()
    }

    let mut page = (**schedule.as_ref().unwrap()).clone();
    page.remove_groups_except(group.to_string());

    Response::from_page(Arc::new(page)).to_json()
}


struct UpdatesWs {
    handle: Option<SpawnHandle>,
}
impl Actor for UpdatesWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut rx = DATA.get().unwrap().schedule.clone().get_notify_rx();

        let stream = async_stream::stream! {
            while rx.changed().await.is_ok() {
                let notify = rx.borrow();
                let str_notify = serde_json::to_string_pretty(&(*notify)).unwrap();

                let msg = ws::Message::Text(str_notify.into());
                
                yield Ok(msg)
            };
        };

        self.handle = Some(ctx.add_stream(stream));
    }
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UpdatesWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context
    ) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(msg)) => ctx.text(msg),
            _ => (),
        }
    }
}

#[get("/schedule/interact")]
async fn interact() -> impl Responder {
    let interactor = {
        DATA.get().unwrap()
        .schedule.clone()
        .new_interactor().await
    };
    Response::from_interactor(interactor).to_json()
}

#[get("/schedule/updates")]
async fn updates(req: HttpRequest, stream: web::Payload) -> impl Responder {
    let update_ws = UpdatesWs {
        handle: None
    };

    let resp = ws::start(update_ws, &req, stream);
    debug!("schedule updates websocket handshake: {:?}", resp);

    resp
}

#[post("/schedule/update?key={key}")]
async fn update(key: web::Path<String>) -> impl Responder {
    DATA.get().unwrap()
    .schedule.index.clone()
    .update_all_manually(key.to_string())
    .await.unwrap();

    Response::ok().to_json()
}