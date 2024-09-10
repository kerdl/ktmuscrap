use actix_web::web::Bytes;
use chrono::Duration;
use log::{info, debug};
use tokio::sync::{RwLock, mpsc, watch};
use std::{path::PathBuf, sync::Arc, collections::HashSet};
use crate::{
    compare::{self, DetailedCmp},
    data::{
        json::Saving,
        schedule::{raw, Last, Notify}
    },
    merge, parse, string, SyncResult
};


#[derive(Debug)]
pub struct Schedule {
    dir: PathBuf,
    updated_rx: Arc<RwLock<mpsc::Receiver<Vec<raw::index::PathHolder>>>>,
    converted_tx: Arc<RwLock<mpsc::Sender<()>>>,

    notify_tx: watch::Sender<Arc<Notify>>,
    notify_rx: watch::Receiver<Arc<Notify>>,

    pub last: Arc<Last>,
    pub index: Arc<raw::Index>
}
impl Schedule {
    pub async fn default_from_dir(dir: PathBuf) -> SyncResult<Arc<Schedule>> {
        if !dir.exists() {
            tokio::fs::create_dir(&dir).await?;
        }

        let (updated_tx, updated_rx) = mpsc::channel(1);
        let (converted_tx, converted_rx) = mpsc::channel(1);
        let (notify_tx, notify_rx) = watch::channel({
            let notify = Notify {
                random: string::random(16),
                groups: None,
                teachers: None,
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
            index: raw::Index::load_or_init(
                dir.join("index.json"),
                updated_tx,
                converted_rx,
            ).await?
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
            let mut rx = self.updated_rx.write().await;
            let paths = rx.recv().await.unwrap();
            debug!("updated signal received");
            std::mem::drop(rx);

            let group_paths = paths.iter()
                .filter(|holder| holder.kind == raw::Kind::Groups)
                .map(|holder| holder.paths.as_slice())
                .flat_map(|paths| paths)
                .cloned()
                .collect::<Vec<PathBuf>>();
            let teacher_paths = paths.iter()
                .filter(|holder| holder.kind == raw::Kind::Teachers)
                .map(|holder| holder.paths.as_slice())
                .flat_map(|paths| paths)
                .cloned()
                .collect::<Vec<PathBuf>>();

            let self_ref = self.clone();
            let groups_handle = tokio::spawn(async move {
                parse::groups(
                    group_paths.as_slice(), self_ref.last.clone()
                ).await.unwrap()
            });
            let self_ref = self.clone();
            let teachers_handle = tokio::spawn(async move {
                parse::teachers(
                    teacher_paths.as_slice(), self_ref.last.clone()
                ).await.unwrap()
            });

            groups_handle.await.unwrap();
            teachers_handle.await.unwrap();
            

            /* 
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

                let ft_path = ft.iter().nth(0).cloned();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::daily(
                    ft_path,
                    Some(r_paths),
                    raw::Mode::Groups,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            if ft_weekly.has_latest().await || r_weekly.has_latest().await {
                let ft = ft_weekly.latest.read().await;
                let r = r_weekly.latest.read().await;

                let ft_path = ft.iter().nth(0).cloned();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::weekly(
                    ft_path,
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

                let ft_path = ft.iter().nth(0).cloned();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::weekly(
                    ft_path,
                    Some(r_paths),
                    raw::Mode::Teachers,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            if tchr_ft_daily.has_latest().await || tchr_r_weekly.has_latest().await {
                let ft = tchr_ft_daily.latest.read().await;
                let r = tchr_r_weekly.latest.read().await;

                let ft_path = ft.iter().nth(0).cloned();
                let r_paths = r.iter().map(|path|path.clone()).collect::<Vec<PathBuf>>();

                parse::daily(
                    ft_path,
                    Some(r_paths),
                    raw::Mode::Teachers,
                    new_last.clone(),
                    new_raw_last.clone()
                ).await.unwrap();
            }

            // remove num_time_mappings
            {
                let mut tchr_daily_new = new_last.tchr_daily.write().await;
                let mut tchr_weekly_new = new_last.tchr_weekly.write().await;

                let mut daily_clone = tchr_daily_new.as_ref().map(|page| (**page).clone());
                let mut weekly_clone = tchr_weekly_new.as_ref().map(|page| (**page).clone());

                daily_clone.as_mut().map(|page| page.num_time_mappings = None);
                weekly_clone.as_mut().map(|page| page.num_time_mappings = None);

                *tchr_daily_new = daily_clone.map(|page| Arc::new(page));
                *tchr_weekly_new = weekly_clone.map(|page| Arc::new(page));
            }

            // merge tchr daily with tchr weekly
            {
                let mut new_tchr_daily = new_last.tchr_daily.write().await;
                let new_tchr_weekly = new_last.tchr_weekly.read().await;
    
                if let (Some(daily), Some(weekly)) = (new_tchr_daily.clone(), new_tchr_weekly.clone()) {
                    let mut new_daily = (*daily).clone();
                    let _ = merge::daily::tchr_ft_page(&mut new_daily, (*weekly).clone()).await;
                    *new_tchr_daily = Some(Arc::new(new_daily));
                }
            }

            // compare old last with new last
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

            // move new last to old last
            {
                debug!("setting last schedules...");

                *self.last.daily.write().await = {
                    new_last.daily.read().await.clone()
                };
                *self.last.weekly.write().await = {
                    new_last.weekly.read().await.clone()
                };

                *self.last.tchr_daily.write().await = {
                    new_last.tchr_daily.read().await.clone()
                };
                *self.last.tchr_weekly.write().await = {
                    new_last.tchr_weekly.read().await.clone()
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
            */

            // sending an event that conversion had finished
            {
                self.converted_tx.read().await.send(()).await.unwrap();
                debug!("converted signal sent");
            }
        }
    }
}
