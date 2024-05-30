use actix_web::web::Bytes;
use std::{path::PathBuf, sync::Arc, collections::HashSet};

use chrono::Duration;
use log::{info, debug};
use tokio::sync::{RwLock, mpsc, watch};

use crate::{
    SyncResult,
    parse, 
    compare::{
        self,
        DetailedCmp
    },
    data::{
        json::Saving,
        schedule::Lifetime
    }, string
};

use super::{
    schedule::{
        raw,
        update,
        Last,
        Notify,
        Interactor
    }};


#[derive(Debug)]
pub struct Schedule {
    dir: PathBuf,
    updated_rx: Arc<RwLock<mpsc::Receiver<update::Params>>>,
    converted_tx: Arc<RwLock<mpsc::Sender<()>>>,

    notify_tx: watch::Sender<Arc<Notify>>,
    notify_rx: watch::Receiver<Arc<Notify>>,

    pub last: Arc<Last>,
    pub raw_last: Arc<raw::Last>,
    pub index: Arc<raw::Index>,

    pub interactors: Arc<RwLock<HashSet<Arc<Interactor>>>>
}
impl Schedule {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Arc<Schedule>> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let (updated_tx, updated_rx)     = mpsc::channel(1024);
        let (converted_tx, converted_rx) = mpsc::channel(1024);
        let (notify_tx, notify_rx)       = watch::channel({
            let notify = Notify {
                random: string::random(16),
                invoker: update::Invoker::Auto,
                daily: None,
                weekly: None,
                tchr_daily: None,
                tchr_weekly: None,
            };

            Arc::new(notify)
        });

        let this = Self {
            dir: dir.clone(),
            updated_rx: Arc::new(RwLock::new(updated_rx)),
            converted_tx: Arc::new(RwLock::new(converted_tx)),

            notify_tx,
            notify_rx,

            last: Last::load_or_init(
                dir.join("last.json")
            ).await?,
            raw_last: raw::Last::load_or_init(
                dir.join("raw_last.json")
            ).await?,
            index: raw::Index::load_or_init(
                crate::FETCH,
                dir.join("index.json"),
                updated_tx,
                converted_rx,
            ).await?,

            interactors: Arc::new(RwLock::new(HashSet::new()))
        };

        let this = Arc::new(this);
        let this_ref = this.clone();

        tokio::spawn(async move {
            this_ref.await_updates().await;
        });

        Ok(this)
    }

    pub fn get_notify_rx(self: Arc<Self>) -> watch::Receiver<Arc<Notify>> {
        self.notify_rx.clone()
    }

    pub async fn await_updates(self: Arc<Self>) {
        loop {
            let params;

            {
                let mut rx = self.updated_rx.write().await;
                params = rx.recv().await.unwrap();

                debug!("updated signal recieved");
            }

            let ft_daily = self.index.ft_daily().await;
            let ft_weekly = self.index.ft_weekly().await;
            let r_weekly = self.index.r_weekly().await;
            let tchr_ft_daily = self.index.tchr_ft_daily().await;
            let tchr_ft_weekly = self.index.tchr_ft_weekly().await;
            let tchr_r_weekly = self.index.tchr_r_weekly().await;

            let new_last = self.last.clone().clone_cleared();
            let new_raw_last = self.raw_last.clone().clone_cleared();

            // groups 

            if ft_daily.has_latest().await || r_weekly.has_latest().await {
                let ft = ft_daily.latest.read().await;
                let r = r_weekly.latest.read().await;

                let ft_path = ft.iter().nth(0).unwrap().clone();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::daily(
                    Some(ft_path),
                    Some(r_paths),
                    raw::Mode::Groups,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            if ft_weekly.has_latest().await || r_weekly.has_latest().await {
                let ft = ft_weekly.latest.read().await;
                let r = r_weekly.latest.read().await;

                let ft_path = ft.iter().nth(0).unwrap().clone();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::weekly(
                    Some(ft_path),
                    Some(r_paths),
                    raw::Mode::Groups,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            // teachers

            if tchr_ft_weekly.has_latest().await || tchr_r_weekly.has_latest().await {
                let ft = tchr_ft_weekly.latest.read().await;
                let r = tchr_r_weekly.latest.read().await;

                let ft_path = ft.iter().nth(0).unwrap().clone();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::weekly(
                    Some(ft_path),
                    Some(r_paths),
                    raw::Mode::Teachers,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            if tchr_ft_daily.has_latest().await || tchr_r_weekly.has_latest().await {
                let ft = tchr_ft_daily.latest.read().await;
                let r = tchr_r_weekly.latest.read().await;

                let ft_path = ft.iter().nth(0).unwrap().clone();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::daily(
                    Some(ft_path),
                    Some(r_paths),
                    raw::Mode::Teachers,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            /* compare old last with new last */
            {
                let daily_old = self.last.daily.read().await;
                let daily_new = new_last.daily.read().await;
                let weekly_old = self.last.weekly.read().await;
                let weekly_new = new_last.weekly.read().await;

                let tchr_daily_old = self.last.tchr_daily.read().await;
                let tchr_daily_new = new_last.tchr_daily.read().await;
                let tchr_weekly_old = self.last.tchr_weekly.read().await;
                let tchr_weekly_new = new_last.tchr_weekly.read().await;

                let daily_changes = compare::schedule::Page::compare(
                    daily_old.as_ref().map(|old| (**old).clone()),
                    daily_new.as_ref().map(|new| (**new).clone()),
                ).await;
                let weekly_changes = compare::schedule::Page::compare(
                    weekly_old.as_ref().map(|old| (**old).clone()),
                    weekly_new.as_ref().map(|new| (**new).clone()),
                ).await;

                let tchr_daily_changes = compare::schedule::TchrPage::compare(
                    tchr_daily_old.as_ref().map(|old| (**old).clone()),
                    tchr_daily_new.as_ref().map(|new| (**new).clone()),
                ).await;
                let tchr_weekly_changes = compare::schedule::TchrPage::compare(
                    tchr_weekly_old.as_ref().map(|old| (**old).clone()),
                    tchr_weekly_new.as_ref().map(|new| (**new).clone()),
                ).await;

                let notify = Notify {
                    random: string::random(16),
                    invoker: params.invoker,
                    daily: if daily_changes.groups.has_changes() {
                        Some(daily_changes)
                    } else {
                        None
                    },
                    weekly: if weekly_changes.groups.has_changes() {
                        Some(weekly_changes)
                    } else {
                        None
                    },
                    tchr_daily: if tchr_daily_changes.teachers.has_changes() {
                        Some(tchr_daily_changes)
                    } else {
                        None
                    },
                    tchr_weekly: if tchr_weekly_changes.teachers.has_changes() {
                        Some(tchr_weekly_changes)
                    } else {
                        None
                    }
                };

                let none = "None".to_owned();

                if notify.daily.is_some() {
                    info!("DAILY CHANGES");
                    info!("   appeared groups {}: {:?}",
                        notify.daily.as_ref().unwrap().groups.appeared.len(),
                        notify.daily.as_ref().unwrap().groups.appeared.iter().map(
                            |group| &group.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   disappeared groups {}: {:?}",
                        notify.daily.as_ref().unwrap().groups.disappeared.len(),
                        notify.daily.as_ref().unwrap().groups.disappeared.iter().map(
                            |group| &group.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   changed groups {}: {:?}",
                        notify.daily.as_ref().unwrap().groups.changed.len(),
                        notify.daily.as_ref().unwrap().groups.changed.iter().map(
                            |group| group.name.as_ref().unwrap_or(&none)
                        ).collect::<Vec<&String>>()
                    );
                }

                if notify.weekly.is_some() {
                    info!("WEEKLY CHANGES");
                    info!("   appeared groups {}: {:?}",
                        notify.weekly.as_ref().unwrap().groups.appeared.len(),
                        notify.weekly.as_ref().unwrap().groups.appeared.iter().map(
                            |group| &group.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   disappeared groups {}: {:?}",
                        notify.weekly.as_ref().unwrap().groups.disappeared.len(),
                        notify.weekly.as_ref().unwrap().groups.disappeared.iter().map(
                            |group| &group.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   changed groups {}: {:?}",
                        notify.weekly.as_ref().unwrap().groups.changed.len(),
                        notify.weekly.as_ref().unwrap().groups.changed.iter().map(
                            |group| group.name.as_ref().unwrap_or(&none)
                        ).collect::<Vec<&String>>()
                    );
                }

                if notify.tchr_daily.is_some() {
                    info!("TCHR DAILY CHANGES");
                    info!("   appeared teachers {}: {:?}",
                        notify.tchr_daily.as_ref().unwrap().teachers.appeared.len(),
                        notify.tchr_daily.as_ref().unwrap().teachers.appeared.iter().map(
                            |teacher| &teacher.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   disappeared teachers {}: {:?}",
                        notify.tchr_daily.as_ref().unwrap().teachers.disappeared.len(),
                        notify.tchr_daily.as_ref().unwrap().teachers.disappeared.iter().map(
                            |teacher| &teacher.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   changed teachers {}: {:?}",
                        notify.tchr_daily.as_ref().unwrap().teachers.changed.len(),
                        notify.tchr_daily.as_ref().unwrap().teachers.changed.iter().map(
                            |teacher| teacher.name.as_ref().unwrap_or(&none)
                        ).collect::<Vec<&String>>()
                    );
                }

                if notify.tchr_weekly.is_some() {
                    info!("TCHR WEEKLY CHANGES");
                    info!("   appeared teachers {}: {:?}",
                        notify.tchr_weekly.as_ref().unwrap().teachers.appeared.len(),
                        notify.tchr_weekly.as_ref().unwrap().teachers.appeared.iter().map(
                            |teacher| &teacher.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   disappeared teachers {}: {:?}",
                        notify.tchr_weekly.as_ref().unwrap().teachers.disappeared.len(),
                        notify.tchr_weekly.as_ref().unwrap().teachers.disappeared.iter().map(
                            |teacher| &teacher.name
                        ).collect::<Vec<&String>>()
                    );
                    info!("   changed teachers {}: {:?}",
                        notify.tchr_weekly.as_ref().unwrap().teachers.changed.len(),
                        notify.tchr_weekly.as_ref().unwrap().teachers.changed.iter().map(
                            |teacher| teacher.name.as_ref().unwrap_or(&none)
                        ).collect::<Vec<&String>>()
                    );
                }

                self.notify_tx.send(Arc::new(notify)).unwrap();
            }

            /* move new last to old last */
            {
                debug!("setting last schedules...");

                *self.last.daily.write().await = {
                    new_last.daily.read().await.clone()
                };
                *self.last.weekly.write().await = {
                    new_last.weekly.read().await.clone()
                };

                self.last.save().await.unwrap();
    

                debug!("setting raw_last schedules...");

                *self.raw_last.ft_daily.write().await = {
                    new_raw_last.ft_daily.read().await.clone()
                };
                *self.raw_last.ft_weekly.write().await = {
                    new_raw_last.ft_weekly.read().await.clone()
                };
                *self.raw_last.r_weekly.write().await = {
                    new_raw_last.r_weekly.read().await.clone()
                };

                *self.raw_last.tchr_ft_daily.write().await = {
                    new_raw_last.tchr_ft_daily.read().await.clone()
                };
                *self.raw_last.tchr_ft_weekly.write().await = {
                    new_raw_last.tchr_ft_weekly.read().await.clone()
                };
                *self.raw_last.tchr_r_weekly.write().await = {
                    new_raw_last.tchr_r_weekly.read().await.clone()
                };

                self.raw_last.save().await.unwrap();
            }

            /* sending an event that conversion had finished */
            {
                self.converted_tx.read().await.send(()).await.unwrap();
                debug!("converted signal sent");
            }
        }
    }

    pub async fn new_interactor(self: Arc<Self>) -> Arc<Interactor> {
        let mut interactors = self.interactors.write().await;

        let (keep_alive_tx, mut keep_alive_rx) = mpsc::channel(1024);
        let (ping_tx, ping_rx) = mpsc::channel(1024);
        let ping_tx = Arc::new(ping_tx);

        let interactor = Interactor::new(keep_alive_tx, ping_rx);

        loop {
            if interactors.insert(interactor.clone()) {
                break;
            }
        }

        debug!("added new interactor {}", interactor.key);


        let self_ref = self.clone();
        let interactor_ref = interactor.clone();
        let ping_tx_ref = ping_tx.clone();
        tokio::spawn(async move {
            loop {
                let self_ref = self_ref.clone();
                let interactor_ref = interactor_ref.clone();
                let ping_tx_ref = ping_tx_ref.clone();

                /* 
                debug!(
                    "spawning destruction handle for {}",
                    interactor_ref.clone().key
                );
                */

                let fuckrust_interactor_ref = interactor_ref.clone();
                // spawn a task to destruct this interactor later
                let destruction_handle = tokio::spawn(async move {
                    // sleep for 10 minutes
                    let dur = Duration::minutes(10).to_std().unwrap();
                    tokio::time::sleep(dur).await;

                    let empty_bytes = Bytes::from(vec![]);

                    if fuckrust_interactor_ref.is_connected().await {
                        // ping interactor 3 times
                        for _ in 0..3 {
                            debug!(
                                "pinging interactor {}",
                                fuckrust_interactor_ref.clone().key
                            );

                            ping_tx_ref.clone().send(
                                empty_bytes.clone()
                            ).await.unwrap();

                            // sleep for 1 second
                            let dur = Duration::seconds(1).to_std().unwrap();
                            tokio::time::sleep(dur).await;
                        }
                    }

                    debug!(
                        "destruction wishes to drop {}",
                        fuckrust_interactor_ref.clone().key
                    );

                    fuckrust_interactor_ref.wish_drop().await;
                });

                match keep_alive_rx.recv().await {
                    Some(Lifetime::Kept) => {
                        /* 
                        debug!(
                            "watch received kept, aborting destruction {}",
                            interactor_ref.clone().key
                        );
                        */

                        destruction_handle.abort();

                        continue;
                    },
                    Some(Lifetime::Drop) => {
                        debug!(
                            "watch received drop {}",
                            interactor_ref.clone().key
                        );

                        destruction_handle.abort();

                        self_ref.clone().remove_interactor(
                            interactor_ref.clone()
                        ).await;
                    },
                    _ => ()
                }

                debug!(
                    "watch ends for {}",
                    interactor_ref.key
                );
                return
            }
        });

        interactors.get(&interactor).unwrap().clone()
    }

    pub async fn get_interactor(&self, key: String) -> Option<Arc<Interactor>> {
        let dummy = Interactor::from_key(key);

        self.interactors.read().await.get(&dummy).map(
            |interactor| interactor.clone()
        )
    }

    async fn remove_interactor(&self, interactor: Arc<Interactor>) {
        self.interactors.write().await.remove(&interactor);
        debug!("removed interactor {}", interactor.key);
    }
}

#[derive(Debug)]
pub struct Container {
    pub dir: PathBuf,

    pub schedule: Arc<Schedule>,
}
impl Container {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Container> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let this = Container {
            dir: dir.clone(),
            schedule: Schedule::default_from_dir(
                dir.join("schedule")
            ).await?
        };

        Ok(this)
    }
}