use actix_web::web::Bytes;
use chrono::Duration;
use log::{info, warn, debug};
use tokio::sync::{RwLock, mpsc, watch};
use std::{path::PathBuf, sync::Arc, collections::HashSet};
use crate::{
    compare::{self, DetailedCmp},
    data::{
        json::Saving,
        schedule::{raw, Last, Notify, Page}
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
                .flatten()
                .cloned()
                .collect::<Vec<PathBuf>>();
            let teacher_paths = paths.iter()
                .filter(|holder| holder.kind == raw::Kind::Teachers)
                .map(|holder| holder.paths.as_slice())
                .flat_map(|paths| paths)
                .cloned()
                .collect::<Vec<PathBuf>>();

            let groups_handle = tokio::spawn(async move {
                parse::groups(group_paths.as_slice()).await
            });
            let teachers_handle = tokio::spawn(async move {
                parse::teachers(teacher_paths.as_slice()).await
            });

            let group_pages = groups_handle
                .await
                .unwrap()
                .into_iter()
                .filter_map(|result| result.ok())
                .collect::<Vec<Page>>();
            let groups_min_date = group_pages
                .iter()
                .min_by(|a, b| a.date.start().cmp(b.date.start()))
                .map(|page| page.date.clone());
            let groups_max_date = group_pages
                .iter()
                .max_by(|a, b| a.date.start().cmp(b.date.start()))
                .map(|page| page.date.clone());
            let groups_date_range = if let (Some(min), Some(max)) = (
                groups_min_date, groups_max_date
            ) {
                Some(min.start().clone()..=max.end().clone())
            } else {
                None
            };

            let teacher_pages = teachers_handle
                .await
                .unwrap()
                .into_iter()
                .filter_map(|result| result.ok())
                .collect::<Vec<Page>>();
            let teachers_min_date = teacher_pages
                .iter()
                .min_by(|a, b| a.date.start().cmp(b.date.start()))
                .map(|page| page.date.clone());
            let teachers_max_date = teacher_pages
                .iter()
                .max_by(|a, b| a.date.start().cmp(b.date.start()))
                .map(|page| page.date.clone());
            let teachers_date_range = if let (Some(min), Some(max)) = (
                teachers_min_date, teachers_max_date
            ) {
                Some(min.start().clone()..=max.end().clone())
            } else {
                None
            };
            
            let mut groups = if let Some(date) = groups_date_range {
                Some(merge::combine(group_pages, date, raw::Kind::Groups))
            } else {
                None
            };
            
            let mut teachers = if let Some(date) = teachers_date_range {
                Some(merge::combine(teacher_pages, date, raw::Kind::Teachers))
            } else {
                None
            };

            if let (Some(groups), Some(teachers)) = (groups.as_mut(), teachers.as_mut()) {
                let result = merge::complement(groups, teachers);
                if let Err(err) = result {
                    warn!("complementing groups with teachers error: {:?}", err);
                }
            }

            let group_changes = compare::schedule::Page::compare(
                self.last.groups.read().await.as_ref().map(|page| (**page).clone()),
                groups.as_ref().cloned()
            ).await;
            let teacher_changes = compare::schedule::Page::compare(
                self.last.teachers.read().await.as_ref().map(|page| (**page).clone()),
                teachers.as_ref().cloned()
            ).await;

            let notify = Notify {
                random: string::random(16),
                groups: if group_changes.formations.has_changes() {
                    Some(group_changes)
                } else {
                    None
                },
                teachers: if teacher_changes.formations.has_changes() {
                    Some(teacher_changes)
                } else {
                    None
                }
            };

            let none_str = "None".to_string();

            if notify.groups.is_some() {
                info!("GROUP CHANGES");
                info!("   appeared groups {}: {:?}",
                    notify.groups.as_ref().unwrap().formations.appeared.len(),
                    notify.groups.as_ref().unwrap().formations.appeared.iter().map(
                        |form| &form.name
                    ).collect::<Vec<&String>>()
                );
                info!("   disappeared groups {}: {:?}",
                    notify.groups.as_ref().unwrap().formations.disappeared.len(),
                    notify.groups.as_ref().unwrap().formations.disappeared.iter().map(
                        |form| &form.name
                    ).collect::<Vec<&String>>()
                );
                info!("   changed groups {}: {:?}",
                    notify.groups.as_ref().unwrap().formations.changed.len(),
                    notify.groups.as_ref().unwrap().formations.changed.iter().map(
                        |form| form.name.as_ref().unwrap_or(&none_str)
                    ).collect::<Vec<&String>>()
                );
            }

            if notify.teachers.is_some() {
                info!("TEACHER CHANGES");
                info!("   appeared teachers {}: {:?}",
                    notify.teachers.as_ref().unwrap().formations.appeared.len(),
                    notify.teachers.as_ref().unwrap().formations.appeared.iter().map(
                        |form| &form.name
                    ).collect::<Vec<&String>>()
                );
                info!("   disappeared teachers {}: {:?}",
                    notify.teachers.as_ref().unwrap().formations.disappeared.len(),
                    notify.teachers.as_ref().unwrap().formations.disappeared.iter().map(
                        |form| &form.name
                    ).collect::<Vec<&String>>()
                );
                info!("   changed teachers {}: {:?}",
                    notify.teachers.as_ref().unwrap().formations.changed.len(),
                    notify.teachers.as_ref().unwrap().formations.changed.iter().map(
                        |form| form.name.as_ref().unwrap_or(&none_str)
                    ).collect::<Vec<&String>>()
                );
            }

            self.notify_tx.send(Arc::new(notify)).unwrap();

            *self.last.groups.write().await = groups.map(|pg| Arc::new(pg));
            *self.last.teachers.write().await = teachers.map(|pg| Arc::new(pg));

            self.last.save().await.unwrap();

            self.converted_tx.read().await.send(()).await.unwrap();
            debug!("converted signal sent");
        }
    }
}
