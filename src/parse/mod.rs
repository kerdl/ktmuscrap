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
    data::{schedule::{
        Last,
        raw
    },
    json::SavingLoading},
};
use super::merge;


fn poll_fulltime(
    dir: PathBuf,
    sc_type: raw::Type,
    raw_last: Arc<raw::Last>
) -> JoinHandle<SyncResult<()>> {

    tokio::spawn(async move {
        let path = raw::fulltime::latest(&dir).await?;

        if path.is_none() {
            return Err(error::NoLatest::new(sc_type).into())
        }

        let path = path.unwrap();

        fulltime::parse_ft_weekly(path, raw_last).await?;

        Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
    })

}

fn poll_remote(
    dir: PathBuf,
    raw_last: Arc<raw::Last>
) -> JoinHandle<SyncResult<()>> {

    tokio::spawn(async move {
        let path = raw::remote::latest(&dir).await?;

        if path.is_none() {
            return Err(error::NoLatest::new(
                raw::Type::RWeekly
            ).into())
        }

        let path = path.unwrap();

        remote::parse(path, raw_last).await?;

        Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
    })

}


pub async fn weekly(
    ft_dir: PathBuf,
    r_dir: PathBuf,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {

    if last.weekly.read().await.is_some() {
        return Ok(())
    }

    let mut parsing_processes = vec![];


    if raw_last.ft_weekly.read().await.is_none() {

        let process = poll_fulltime(
            ft_dir,
            raw::Type::FtWeekly,
            raw_last.clone()
        );

        parsing_processes.push(process);
    }
    
    if raw_last.r_weekly.read().await.is_none() {

        let process = poll_remote(
            r_dir,
            raw_last.clone()
        );

        parsing_processes.push(process);
    }

    for process in parsing_processes {
        process.await??;
    }

    // yes, actually clone large
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

    *last.weekly.write().await = {
        Some(Arc::new(ft_weekly_page))
    };
    last.poll_save();

    Ok(())
}

pub async fn daily(
    ft_dir: PathBuf,
    r_dir: PathBuf,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {

    if last.daily.read().await.is_some() {
        return Ok(())
    }

    let mut parsing_processes = vec![];


    if raw_last.ft_daily.read().await.is_none() {

        let process = poll_fulltime(
            ft_dir,
            raw::Type::FtDaily,
            raw_last.clone()
        );

        parsing_processes.push(process);
    }
    
    if raw_last.r_weekly.read().await.is_none() {

        let process = poll_remote(
            r_dir,
            raw_last.clone()
        );

        parsing_processes.push(process);
    }

    for process in parsing_processes {
        process.await??;
    }

    // yes, actually clone large
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

    *last.daily.write().await = {
        Some(Arc::new(ft_daily_page))
    };
    last.poll_save();

    Ok(())
}
