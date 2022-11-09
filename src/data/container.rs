use std::{path::PathBuf, sync::Arc};

use log::{info, debug};
use tokio::sync::{RwLock, mpsc, watch};

use crate::{SyncResult, parse, compare::{self, DetailedCmp}, data::json::Saving};

use super::{schedule::{Last, raw, Notify}};


#[derive(Debug)]
pub struct Schedule {
    dir: PathBuf,
    updated_rx: Arc<RwLock<mpsc::Receiver<()>>>,
    converted_tx: Arc<RwLock<mpsc::Sender<()>>>,

    notify_tx: watch::Sender<Notify>,
    notify_rx: watch::Receiver<Notify>,

    pub last: Arc<Last>,
    pub raw_last: Arc<raw::Last>,
    pub index: Arc<raw::Index>
}
impl Schedule {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Arc<Schedule>> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let (updated_tx, updated_rx)     = mpsc::channel(1024);
        let (converted_tx, converted_rx) = mpsc::channel(1024);
        let (notify_tx, notify_rx)       = watch::channel(
            Notify {
                daily: None,
                weekly: None
            }
        );

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
                dir.join("index.json"),
                updated_tx,
                converted_rx,
            ).await?,
        };

        let this = Arc::new(this);
        let this_ref = this.clone();

        tokio::spawn(async move {
            this_ref.await_updates().await;
        });

        Ok(this)
    }

    pub fn get_notify_rx(self: Arc<Self>) -> watch::Receiver<Notify> {
        self.notify_rx.clone()
    }

    pub async fn await_updates(self: Arc<Self>) {
        loop {
            {
                let mut rx = self.updated_rx.write().await;
                rx.recv().await.unwrap();

                debug!("updated signal recieved");
            }

            let ft_daily = self.index.types.iter().find(
                |schedule| schedule.sc_type == raw::Type::FtDaily
            ).unwrap();
            let ft_weekly = self.index.types.iter().find(
                |schedule| schedule.sc_type == raw::Type::FtWeekly
            ).unwrap();
            let r_weekly = self.index.types.iter().find(
                |schedule| schedule.sc_type == raw::Type::RWeekly
            ).unwrap();

            let new_last = self.last.clone().clone_cleared();
            let new_raw_last = self.raw_last.clone().clone_cleared();

            if ft_daily.has_latest().await || r_weekly.has_latest().await {
                parse::daily(
                    ft_daily.latest.read().await.clone(),
                    r_weekly.latest.read().await.clone(),
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            if ft_weekly.has_latest().await || r_weekly.has_latest().await {
                parse::weekly(
                    ft_weekly.latest.read().await.clone(),
                    r_weekly.latest.read().await.clone(),
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
    
                let daily_changes = compare::schedule::Page::compare(
                    daily_old.as_ref().map(|old| (**old).clone()),
                    (**daily_new.as_ref().unwrap()).clone()
                ).await;
                let weekly_changes = compare::schedule::Page::compare(
                    weekly_old.as_ref().map(|old| (**old).clone()),
                    (**weekly_new.as_ref().unwrap()).clone()
                ).await;


                if {
                    daily_changes.groups.has_changes()
                    || weekly_changes.groups.has_changes()
                } {
                    let notify = Notify {
                        daily: if daily_changes.groups.has_changes() {
                            Some(daily_changes)
                        } else {
                            None
                        },
                        weekly: if weekly_changes.groups.has_changes() {
                            Some(weekly_changes)
                        } else {
                            None
                        }
                    };

                    if notify.daily.is_some() {
                        debug!("DAILY CHANGES");
                        debug!("   appeared groups {}",
                            notify.daily.as_ref().unwrap().groups.appeared.len()
                        );
                        debug!("   disappeared groups {}",
                            notify.daily.as_ref().unwrap().groups.disappeared.len()
                        );
                        debug!("   changed groups {}",
                            notify.daily.as_ref().unwrap().groups.changed.len()
                        );
                        debug!("   unchanged groups {}",
                            notify.daily.as_ref().unwrap().groups.unchanged.len()
                        );
                    }

                    if notify.weekly.is_some() {
                        debug!("WEEKLY CHANGES");
                        debug!("   appeared groups {}",
                            notify.weekly.as_ref().unwrap().groups.appeared.len()
                        );
                        debug!("   disappeared groups {}",
                            notify.weekly.as_ref().unwrap().groups.disappeared.len()
                        );
                        debug!("   changed groups {}",
                            notify.weekly.as_ref().unwrap().groups.changed.len()
                        );
                        debug!("   unchanged groups {}",
                            notify.weekly.as_ref().unwrap().groups.unchanged.len()
                        );
                    }

                    self.notify_tx.send(notify).unwrap();

                    debug!("change notify sent");
                } else {
                    debug!("no changes found");
                }
            }

            /* move new last to old last */
            {
                if self.last.clone().is_cleared().await {
                    debug!("setting last schedules...");
    
                    *self.last.daily.write().await = {
                        new_last.daily.read().await.clone()
                    };
                    *self.last.weekly.write().await = {
                        new_last.weekly.read().await.clone()
                    };
    
                    self.last.save().await.unwrap();
                }
    
                if self.raw_last.clone().is_cleared().await {
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
    
                    self.raw_last.save().await.unwrap();
                }
            }

            /* sending an event that conversion had finished */
            {
                self.converted_tx.read().await.send(()).await.unwrap();
                debug!("converted signal sent");
            }
        }
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