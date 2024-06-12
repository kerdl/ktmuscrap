pub mod weekly;
pub mod daily;
pub mod tchr_weekly;
pub mod tchr_daily;
pub mod raw;

use serde_derive::Deserialize;
use actix::{Actor, StreamHandler, SpawnHandle, AsyncContext};
use actix_web::{web::{self, Bytes}, Responder, get, post, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use log::debug;
use std::sync::Arc;

use crate::{DATA, data::schedule::{Type, Interactor, update::Invoker}};
use super::{error::{self, base::ToApiError}, ToResponse, Response};



#[derive(Deserialize)]
struct InteractionQuery {
    key: String,
}

#[derive(Deserialize)]
struct ScheduleGetQuery {
    group: Option<String>
}

#[derive(Deserialize)]
struct TchrScheduleGetQuery {
    teacher: Option<String>
}


async fn generic_get(sc_type: Type) -> HttpResponse {
    let schedule = match sc_type {
        Type::Weekly => DATA.get().unwrap().schedule.last.weekly.read().await,
        Type::Daily => DATA.get().unwrap().schedule.last.daily.read().await
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

async fn generic_tchr_get(sc_type: Type) -> HttpResponse {
    let schedule = match sc_type {
        Type::Weekly => DATA.get().unwrap().schedule.last.tchr_weekly.read().await,
        Type::Daily => DATA.get().unwrap().schedule.last.tchr_daily.read().await
    };

    if schedule.is_none() {
        return error::NoLastSchedule::new(sc_type)
            .to_api_error()
            .to_response()
            .to_json()
    }

    Response::from_tchr_page(
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

async fn generic_teacher_get(sc_type: Type, teacher: String) -> HttpResponse {
    let schedule = match sc_type {
        Type::Weekly => DATA.get().unwrap().schedule.last.tchr_weekly.read().await,
        Type::Daily  => DATA.get().unwrap().schedule.last.tchr_daily.read().await
    };

    if schedule.is_none() {
        return error::NoLastSchedule::new(sc_type)
            .to_api_error()
            .to_response()
            .to_json()
    }

    let mut page = (**schedule.as_ref().unwrap()).clone();
    page.remove_teachers_except(teacher.to_string());

    Response::from_tchr_page(Arc::new(page)).to_json()
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

                match &notify.invoker {
                    Invoker::Auto => {},
                    Invoker::Manually(invoker) => {
                        if invoker == &interactor {
                            continue;
                        }
                    }
                };

                let msg = serde_json::to_string_pretty(&(*notify)).unwrap();
                
                debug!("sending updates to interactor {}", interactor.key);
                yield msg
            };
        };

        let interactor = self.interactor.clone();

        let ping_stream = async_stream::stream! {
            let mut ping_rx = interactor.ping_rx.as_ref().unwrap().write().await;

            while let Some(bytes) = ping_rx.recv().await {
                yield bytes
            };
        };

        self.updates_handle = Some(ctx.add_stream(updates_stream));
        self.ping_handle = Some(ctx.add_stream(ping_stream));
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
        let interactor = self.interactor.clone();

        //debug!("interactor {} sent message {:?}", interactor.key, msg);

        tokio::spawn(async move {
            interactor.keep_alive().await;
        });

        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            //Ok(ws::Message::Text(msg)) => ctx.text(msg),
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

#[post("/schedule/interact/keep-alive")]
async fn interact_keep_alive(
    query: web::Query<InteractionQuery>,
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
        
        return err;
    }
    let interactor = interactor.unwrap();

    interactor.keep_alive().await;

    Response::ok().to_json()
}

#[get("/schedule/interact/is-valid")]
async fn key_is_valid(
    query: web::Query<InteractionQuery>,
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
        
        return err;
    }

    Response::ok().to_json()
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
    let data = DATA.get().unwrap();
    let key = query.key.clone();

    let interactor = {
        data.schedule
        .get_interactor(key.to_string()).await
    };
    if interactor.is_none() {
        return error::NoSuchKey::new(key.to_string())
            .to_api_error()
            .to_response()
            .to_json()
    }
    let interactor = interactor.unwrap();

    interactor.keep_alive().await;

    data.schedule.index.clone()
        .update_all_manually(interactor)
        .await;

    let notify_rx = data.schedule.clone().get_notify_rx();

    let notify = (*notify_rx.borrow()).clone();

    Response::from_notify(notify).to_json()
}

#[get("/schedule/update/period")]
async fn update_period() -> impl Responder {
    let data = DATA.get().unwrap();

    Response::from_period(
        data.schedule.index.period.to_std().unwrap()
    ).to_json()
}

#[get("/schedule/update/last")]
async fn update_last() -> impl Responder {
    let data = DATA.get().unwrap();

    Response::from_last_update(
        data.schedule.index.updated.read().await.clone()
    ).to_json()
}