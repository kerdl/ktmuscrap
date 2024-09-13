pub mod teachers;
pub mod groups;

use serde_derive::Deserialize;
use actix::{Actor, StreamHandler, SpawnHandle, AsyncContext};
use actix_web::{web::{self, Bytes}, Responder, get, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::debug;
use std::sync::Arc;

use crate::{options, data::schedule::raw::Kind};
use super::{error::{self, base::ToApiError}, ToResponse, Response};


#[derive(Deserialize)]
struct ScheduleGetNameQuery {
    name: Option<String>
}

async fn generic_get(
    kind: Kind,
    query: web::Query<ScheduleGetNameQuery>
) -> HttpResponse {
    let page_guard = match kind {
        Kind::Groups => options().schedule.last.groups.read().await,
        Kind::Teachers => options().schedule.last.teachers.read().await
    };

    let Some(mut page) = page_guard.clone() else {
        return error::NoLastSchedule::new(kind)
            .to_api_error()
            .to_response()
            .to_json()
    };

    if let Some(name) = query.name.as_ref() {
        let mut cloned_page = (*page).clone();
        cloned_page.remove_except(name);
        page = Arc::new(cloned_page);
    }

    Response::from_page(
        page.clone()
    ).to_json()
}

struct UpdatesWs {
    updates_handle: Option<SpawnHandle>
}
impl Actor for UpdatesWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let updates_stream = async_stream::stream! {
            let mut notify_rx = options().schedule.clone().get_notify_rx();

            while notify_rx.changed().await.is_ok() {
                let notify = notify_rx.borrow();
                let msg = serde_json::to_string_pretty(&(*notify)).unwrap();
                debug!("sending updates to subscriber");
                yield msg
            };
        };

        self.updates_handle = Some(ctx.add_stream(updates_stream));
    }
}
impl StreamHandler<String> for UpdatesWs {
    fn handle(
        &mut self,
        item: String,
        ctx: &mut Self::Context
    ) {
        ctx.text(item)
    }
}
impl StreamHandler<Bytes> for UpdatesWs {
    fn handle(
        &mut self,
        item: Bytes,
        ctx: &mut Self::Context
    ) {
        ctx.binary(item)
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
            _ => (),
        }
    }
}

#[get("/schedule/updates")]
async fn updates(
    req: HttpRequest,
    stream: web::Payload
) -> impl Responder {
    let updates_ws = UpdatesWs {
        updates_handle: None
    };

    let resp = ws::start(updates_ws, &req, stream);
    debug!("responding to schedule updates subscriber: {:?}", resp);

    resp
}

#[get("/schedule/updates/period")]
async fn updates_period() -> impl Responder {
    Response::from_updates_period(
        options().schedule.index.period.to_std().unwrap()
    ).to_json()
}

#[get("/schedule/updates/last")]
async fn updates_last() -> impl Responder {
    Response::from_updates_last(
        options().schedule.index.updated.read().await.clone()
    ).to_json()
}