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

use std::{sync::Arc, path::PathBuf, ops::ControlFlow};

use log::warn;
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
) -> JoinHandle<Result<(), fulltime::GenericParsingError>> {

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

        Ok::<(), fulltime::GenericParsingError>(())
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

async fn handle_fail(
    ft_failed: bool,
    r_failed: bool,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> ControlFlow<()> {
    if ft_failed && !r_failed {
        let r_page = raw_last.r_weekly.read().await.as_ref().unwrap().clone();
        last.set_weekly((*r_page).clone()).await;

        return ControlFlow::Break(())
    } else if r_failed && !ft_failed {
        let ft_page = raw_last.ft_weekly.read().await.as_ref().unwrap().clone();
        last.set_weekly((*ft_page).clone()).await;

        return ControlFlow::Break(())
    } else if r_failed && ft_failed {
        return ControlFlow::Break(())
    }

    ControlFlow::Continue(())
}

pub async fn weekly(
    ft_weekly: Option<PathBuf>,
    r_weekly: Option<PathBuf>,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    let mut ft_process = None;
    let mut r_process = None;

    let ft_weekly_converted = raw_last.clone().ft_weekly_is_some().await;
    let r_weekly_converted = raw_last.clone().r_weekly_is_some().await;

    if !ft_weekly_converted && ft_weekly.is_some() {
        ft_process = Some(poll_fulltime(
            ft_weekly.as_ref().unwrap().clone(),
            raw::Type::FtWeekly,
            raw_last.clone()
        ));
    }

    if !r_weekly_converted && r_weekly.is_some() {
        r_process = Some(poll_remote(
            r_weekly.as_ref().unwrap().clone(),
            raw_last.clone()
        ));
    }

    let mut ft_result = None;
    let mut r_result = None;

    let mut ft_failed = false;
    let mut r_failed = false;

    if let Some(ft) = ft_process {
        let result = ft.await.unwrap();

        if result.is_err() {
            warn!("fulltime parsing error: {:?}", result);
        }

        ft_result = Some(result);
    }

    if let Some(r) = r_process {
        let result = r.await.unwrap();

        if result.is_err() {
            warn!("remote parsing error: {:?}", result);
        }

        r_result = Some(result);
    }

    if !ft_weekly_converted && (ft_result.is_none() || ft_result.unwrap().is_err()) {
        ft_failed = true;
    }

    if !r_weekly_converted && (r_result.is_none() || r_result.unwrap().is_err()) {
        r_failed = true;
    }

    match handle_fail(ft_failed, r_failed, last.clone(), raw_last.clone()).await {
        ControlFlow::Break(_) => return Ok(()),
        ControlFlow::Continue(_) => ()
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
    let mut ft_process = None;
    let mut r_process = None;

    if raw_last.clone().ft_daily_is_none().await && ft_daily.is_some() {
        ft_process = Some(poll_fulltime(
            ft_daily.as_ref().unwrap().clone(),
            raw::Type::FtDaily,
            raw_last.clone()
        ));
    }

    if raw_last.clone().r_weekly_is_none().await && r_weekly.is_some() {
        r_process = Some(poll_remote(
            r_weekly.as_ref().unwrap().clone(),
            raw_last.clone()
        ));
    }

    let mut ft_result = None;
    let mut r_result = None;

    let mut ft_failed = false;
    let mut r_failed = false;

    if let Some(ft) = ft_process {
        let result = ft.await.unwrap();

        if result.is_err() {
            warn!("fulltime parsing error: {:?}", result);
        }

        ft_result = Some(result);
    }

    if let Some(r) = r_process {
        let result = r.await.unwrap();

        if result.is_err() {
            warn!("remote parsing error: {:?}", result);
        }

        r_result = Some(result);
    }

    if ft_daily.is_none() || ft_result.unwrap().is_err() {
        ft_failed = true;
    }

    if r_weekly.is_none() || r_result.unwrap().is_err() {
        r_failed = true;
    }

    match handle_fail(ft_failed, r_failed, last.clone(), raw_last.clone()).await {
        ControlFlow::Break(_) => return Ok(()),
        ControlFlow::Continue(_) => ()
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
                    group.remove_days_except(r_weekly_page.date.start());
                }

                r_weekly_page
            },
            _ => unreachable!()
        }
    }

    last.set_daily(ft_daily_page).await;

    Ok(())
}
