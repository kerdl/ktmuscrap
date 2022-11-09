pub mod weekly;
pub mod daily;

use serde_derive::Deserialize;
use actix::{Actor, StreamHandler, SpawnHandle, AsyncContext};
use actix_web::{web::{self, Bytes}, Responder, get, post, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use log::{info, debug};
use tokio::sync::watch;
use std::sync::Arc;

use crate::{DATA, data::schedule::{self, Type, Interactor}, string};
use super::{error::{self, base::ToApiError}, ToResponse, Response};



#[derive(Deserialize)]
struct InteractionQuery {
    key: String,
}

#[derive(Deserialize)]
struct ScheduleGetQuery {
    group: Option<String>
}


async fn generic_get(sc_type: Type) -> HttpResponse {
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

async fn generic_group_get(sc_type: Type, group: String) -> HttpResponse {
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
    updates_handle: Option<SpawnHandle>,
    ping_handle: Option<SpawnHandle>,
    interactor: Arc<Interactor>,
}
impl Actor for UpdatesWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

        let data = DATA.get().unwrap();
        let interactor = self.interactor.clone();

        let updates_stream = async_stream::stream! {
            let mut notify_rx = data.schedule.clone().get_notify_rx();

            while notify_rx.changed().await.is_ok() {
                let notify = notify_rx.borrow();
                let str_notify = serde_json::to_string_pretty(&(*notify)).unwrap();

                let msg = ws::Message::Text(str_notify.into());
                
                yield Ok(msg)
            };
        };

        let ping_stream = async_stream::stream! {
            let mut ping_rx = interactor.ping_rx.as_ref().unwrap().write().await;

            while let Some(bytes) = ping_rx.recv().await {
                let msg = ws::Message::Ping(bytes);
                yield Ok(msg)
            };
        };

        self.updates_handle = Some(ctx.add_stream(updates_stream));
        self.ping_handle = Some(ctx.add_stream(ping_stream));
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let data = DATA.get().unwrap();
        let interactor = self.interactor.clone();

        tokio::spawn(async move {
            interactor.wish_drop().await;
        });
    }
}
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UpdatesWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context
    ) {
        let interactor = self.interactor.clone();

        tokio::spawn(async move {
            interactor.keep_alive().await;
        });

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

    Response::from_interactor(
        interactor.clone()
    ).to_json()
}

#[get("/schedule/updates")]
async fn updates(
    query: web::Query<InteractionQuery>,
    req: HttpRequest,
    stream: web::Payload
) -> impl Responder {
    let key = query.key.clone();

    let interactor = {
        DATA.get().unwrap()
        .schedule.get_interactor(key.to_string()).await
    };
    if interactor.is_none() {
        let err = error::NoSuchKey::new(key.to_string())
            .to_api_error()
            .to_response()
            .to_json();
        
        return Ok(err);
    }
    let interactor = interactor.unwrap();

    interactor.keep_alive().await;

    let updates_ws = UpdatesWs {
        updates_handle: None,
        ping_handle: None,
        interactor: interactor.clone()
    };

    let resp = ws::start(updates_ws, &req, stream);
    debug!("schedule updates websocket handshake: {:?}", resp);

    interactor.connected().await;
    debug!("interactor {} connected", interactor.key);

    resp
}

#[post("/schedule/update")]
async fn update(
    query: web::Query<InteractionQuery>
) -> impl Responder {
    let key = query.key.clone();

    let interactor = {
        DATA.get().unwrap()
        .schedule.get_interactor(key.to_string()).await
    };
    if interactor.is_none() {
        return error::NoSuchKey::new(key.to_string())
            .to_api_error()
            .to_response()
            .to_json()
    }
    let interactor = interactor.unwrap();

    interactor.keep_alive().await;

    DATA.get().unwrap()
        .schedule.index.clone()
        .update_all_manually(key.to_string())
        .await.unwrap();

    Response::ok().to_json()
}