pub mod fulltime;
pub mod remote;
pub mod node;
pub mod date;
pub mod time;
pub mod num;
pub mod group;
pub mod subject;
pub mod teacher;
pub mod cabinet;
pub mod error;

use std::{sync::Arc, path::PathBuf};

use tokio::task::JoinHandle;

use crate::{
    SyncResult, 
    api::error::{
        self as api_err, 
        base::{
            ToApiError, 
            ApiError
        }
    }, 
    data::schedule::{
        Last,
        raw
    },
};
use super::merge;


fn poll_fulltime(
    path: PathBuf,
    sc_type: raw::Type,
    raw_last: Arc<raw::Last>
) -> JoinHandle<SyncResult<()>> {

    tokio::spawn(async move {
        match sc_type {
            raw::Type::FtDaily => {
                fulltime::parse_ft_daily(path, raw_last).await?
            }
            raw::Type::FtWeekly => {
                fulltime::parse_ft_weekly(path, raw_last).await?
            }
            _ => unreachable!()
        }

        Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
    })

}

fn poll_remote(
    path: PathBuf,
    raw_last: Arc<raw::Last>
) -> JoinHandle<SyncResult<()>> {

    tokio::spawn(async move {
        remote::parse(path, raw_last).await?;

        Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
    })

}


pub async fn weekly(
    ft_weekly: Option<PathBuf>,
    r_weekly: Option<PathBuf>,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {

    let mut parsing_processes = vec![];


    if raw_last.clone().ft_weekly_is_none().await && ft_weekly.is_some() {
        let ft_process = poll_fulltime(
            ft_weekly.unwrap(),
            raw::Type::FtWeekly,
            raw_last.clone()
        );
        parsing_processes.push(ft_process);
    }

    if raw_last.clone().r_weekly_is_none().await && r_weekly.is_some() {
        let r_process = poll_remote(
            r_weekly.unwrap(),
            raw_last.clone()
        );
        parsing_processes.push(r_process);
    }


    for process in parsing_processes {
        process.await??;
    }

    // yes, actually clone
    // large `Page` struct
    let mut ft_weekly_page = {
        let arc_page = {
            raw_last.ft_weekly
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };
    let mut r_weekly_page = {
        let arc_page = {
            raw_last.r_weekly
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };

    if let Err(different_weeks) = merge::weekly::page(
        &mut ft_weekly_page, 
        &mut r_weekly_page
    ).await {
        ft_weekly_page = match different_weeks.latest {
            raw::Type::FtWeekly => ft_weekly_page,
            raw::Type::RWeekly =>  r_weekly_page,
            _ => unreachable!()
        }
    }

    last.set_weekly(ft_weekly_page).await;

    Ok(())
}

pub async fn daily(
    ft_daily: Option<PathBuf>,
    r_weekly:Option<PathBuf>,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {

    let mut parsing_processes = vec![];


    if raw_last.clone().ft_daily_is_none().await && ft_daily.is_some() {
        let process = poll_fulltime(
            ft_daily.unwrap(),
            raw::Type::FtDaily,
            raw_last.clone()
        );
        parsing_processes.push(process);
    }

    if raw_last.clone().r_weekly_is_none().await && r_weekly.is_some() {
        let process = poll_remote(
            r_weekly.unwrap(),
            raw_last.clone()
        );
        parsing_processes.push(process);
    }


    for process in parsing_processes {
        process.await??;
    }

    // yes, actually clone
    // large `Page` struct
    let mut ft_daily_page = {
        let arc_page = {
            raw_last.ft_daily
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };
    let mut r_weekly_page = {
        let arc_page = {
            raw_last.r_weekly
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };

    if let Err(ft_not_in_r_range) = merge::daily::page(
        &mut ft_daily_page,
        &mut r_weekly_page
    ).await {
        ft_daily_page = match ft_not_in_r_range.latest {
            raw::Type::FtDaily => { ft_daily_page },
            raw::Type::RWeekly => {
                for group in r_weekly_page.groups.iter_mut() {
                    // remain only first day of the week
                    group.remove_days_except(r_weekly_page.date.start);
                }

                r_weekly_page
            },
            _ => unreachable!()
        }
    }

    last.set_daily(ft_daily_page).await;

    Ok(())
}
