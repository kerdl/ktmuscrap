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
        raw,
        Type
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
    sc_type: Type,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> ControlFlow<()> {
    if ft_failed && !r_failed {
        let r_page = raw_last.r_weekly.read().await.as_ref().unwrap().clone();
        last.set_weekly((*r_page).clone()).await;

        return ControlFlow::Break(())
    } else if r_failed && !ft_failed {
        match sc_type {
            Type::Weekly => {
                let page = raw_last.ft_weekly.read().await.as_ref().unwrap().clone();
                last.set_weekly((*page).clone()).await;
            },
            Type::Daily => {
                let page = raw_last.ft_daily.read().await.as_ref().unwrap().clone();
                last.set_daily((*page).clone()).await;
            }
        };

        return ControlFlow::Break(())
    } else if r_failed && ft_failed {
        return ControlFlow::Break(())
    }

    ControlFlow::Continue(())
}

async fn generic(
    ft: Option<PathBuf>,
    r: Option<PathBuf>,
    sc_type: Type,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    let mut ft_process = None;
    let mut r_process = None;

    let ft_converted = match sc_type {
        Type::Weekly => raw_last.clone().ft_weekly_is_some().await,
        Type::Daily => raw_last.clone().ft_daily_is_some().await,
    };
    let r_converted = raw_last.clone().r_weekly_is_some().await;

    if !ft_converted && ft.is_some() {
        ft_process = Some(poll_fulltime(
            ft.as_ref().unwrap().clone(),
            match sc_type {
                Type::Weekly => raw::Type::FtWeekly,
                Type::Daily => raw::Type::FtDaily
            },
            raw_last.clone()
        ));
    }

    if !r_converted && r.is_some() {
        r_process = Some(poll_remote(
            r.as_ref().unwrap().clone(),
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
            warn!("fulltime {} parsing error: {:?}", sc_type, result);
        }

        ft_result = Some(result);
    }

    if let Some(r) = r_process {
        let result = r.await.unwrap();

        if result.is_err() {
            warn!("remote {} parsing error: {:?}", sc_type, result);
        }

        r_result = Some(result);
    }

    if !ft_converted && (ft_result.is_none() || ft_result.unwrap().is_err()) {
        ft_failed = true;
    }

    if !r_converted && (r_result.is_none() || r_result.unwrap().is_err()) {
        r_failed = true;
    }

    match handle_fail(ft_failed, r_failed, sc_type.clone(), last.clone(), raw_last.clone()).await {
        ControlFlow::Break(_) => return Ok(()),
        ControlFlow::Continue(_) => ()
    }

    // yes, actually clone
    // large `Page` struct
    let ft_page = {
        let arc_page = match sc_type {
            Type::Weekly => raw_last.ft_weekly.read().await.clone(),
            Type::Daily => raw_last.ft_daily.read().await.clone(),
        };
        arc_page.map(|arc_page| (*arc_page).clone())
    };
    let r_page = {
        let arc_page = raw_last.r_weekly.read().await.clone();
        arc_page.map(|arc_page| (*arc_page).clone())
    };

    if ft_page.is_some() && r_page.is_none() {
        let ft_page = ft_page.unwrap();
        last.set_weekly(ft_page).await;
    } else if ft_page.is_none() && r_page.is_some() {
        let r_page = r_page.unwrap();
        last.set_weekly(r_page).await;
    } else if ft_page.is_some() && r_page.is_some() {
        let mut ft_page = ft_page.unwrap();
        let mut r_page = r_page.unwrap();

        match sc_type {
            Type::Weekly => {
                if let Err(different_weeks) = merge::weekly::page(
                    &mut ft_page, 
                    &mut r_page
                ).await {
                    ft_page = match different_weeks.latest {
                        raw::Type::FtWeekly => ft_page,
                        raw::Type::RWeekly => r_page,
                        _ => unreachable!()
                    }
                }
            
                last.set_weekly(ft_page).await;
            },
            Type::Daily => {
                if let Err(ft_not_in_r_range) = merge::daily::page(
                    &mut ft_page,
                    &mut r_page
                ).await {
                    ft_page = match ft_not_in_r_range.latest {
                        raw::Type::FtDaily => { ft_page },
                        raw::Type::RWeekly => {
                            for group in r_page.groups.iter_mut() {
                                // remain only first day of the week
                                group.remove_days_except(r_page.date.start());
                            }

                            r_page.groups.retain(|grp| !grp.days.is_empty());
            
                            r_page
                        },
                        _ => unreachable!()
                    }
                }
            
                last.set_daily(ft_page).await
            }
        }
    }

    Ok(())
}

pub async fn weekly(
    ft_weekly: Option<PathBuf>,
    r_weekly: Option<PathBuf>,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    generic(ft_weekly, r_weekly, Type::Weekly, last, raw_last).await
}

pub async fn daily(
    ft_daily: Option<PathBuf>,
    r_weekly: Option<PathBuf>,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    generic(ft_daily, r_weekly, Type::Daily, last, raw_last).await
}
