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
use std::{collections::HashMap, ops::Range};

use log::warn;
use tokio::task::JoinHandle;
use chrono::NaiveTime;

use crate::data::Weekday;
use crate::{
    SyncResult, 
    data::schedule::{
        Last,
        raw,
        Type,
    },
};
use super::merge;


fn poll_fulltime(
    path: PathBuf,
    sc_type: raw::Type,
    raw_last: Arc<raw::Last>,
    num_time_mappings: Option<HashMap<Weekday, HashMap<u32, Range<NaiveTime>>>>,
) -> JoinHandle<Result<(), fulltime::GenericParsingError>> {
    tokio::spawn(async move {
        match sc_type {
            raw::Type::FtDaily => fulltime::parse_ft_daily(path, raw_last).await?,
            raw::Type::FtWeekly => fulltime::parse_ft_weekly(path, raw_last).await?,
            raw::Type::TchrFtDaily => fulltime::parse_tchr_ft_daily(path, raw_last, num_time_mappings).await?,
            raw::Type::TchrFtWeekly => fulltime::parse_tchr_ft_weekly(path, raw_last, num_time_mappings).await?,
            _ => unreachable!()
        }

        Ok::<(), fulltime::GenericParsingError>(())
    })
}

async fn await_remote(
    paths: Vec<PathBuf>,
    sc_mode: raw::Mode,
    raw_last: Arc<raw::Last>
) -> SyncResult<()> {
    match sc_mode {
        raw::Mode::Groups => {
            if paths.is_empty() {
                return Ok(());
            }
            remote::parse(paths.get(0).unwrap().clone(), raw_last).await
        },
        raw::Mode::Teachers => remote::tchr_parse(paths.as_slice(), raw_last).await
    }
}

fn poll_remote(
    paths: Vec<PathBuf>,
    sc_mode: raw::Mode,
    raw_last: Arc<raw::Last>
) -> JoinHandle<SyncResult<()>> {
    tokio::spawn(async move {
        await_remote(paths, sc_mode, raw_last).await
    })
}

async fn handle_fail(
    ft_failed: bool,
    r_failed: bool,
    sc_type: Type,
    sc_mode: raw::Mode,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> ControlFlow<()> {
    if ft_failed && !r_failed {
        match sc_mode {
            raw::Mode::Groups => match sc_type {
                Type::Weekly => {
                    let r_page = raw_last.r_weekly.read().await.as_ref().unwrap().clone();
                    last.set_weekly((*r_page).clone()).await;
                },
                Type::Daily => {

                },
            },
            raw::Mode::Teachers => match sc_type {
                Type::Weekly => {
                    let r_page = raw_last.tchr_r_weekly.read().await.as_ref().unwrap().clone();
                    last.set_tchr_weekly((*r_page).clone()).await;
                },
                Type::Daily => {

                },
            }
        }

        return ControlFlow::Break(())
    } else if r_failed && !ft_failed {
        match sc_mode {
            raw::Mode::Groups => match sc_type {
                Type::Weekly => {
                    let page = raw_last.ft_weekly.read().await.as_ref().unwrap().clone();
                    last.set_weekly((*page).clone()).await;
                },
                Type::Daily => {
                    let page = raw_last.ft_daily.read().await.as_ref().unwrap().clone();
                    last.set_daily((*page).clone()).await;
                }
            },
            raw::Mode::Teachers => match sc_type {
                Type::Weekly => {
                    let page = raw_last.tchr_ft_weekly.read().await.as_ref().unwrap().clone();
                    last.set_tchr_weekly((*page).clone()).await;
                },
                Type::Daily => {
                    let page = raw_last.tchr_ft_daily.read().await.as_ref().unwrap().clone();
                    last.set_tchr_daily((*page).clone()).await;
                }
            }
        }

        return ControlFlow::Break(())
    } else if r_failed && ft_failed {
        return ControlFlow::Break(())
    }

    ControlFlow::Continue(())
}

async fn generic(
    ft: Option<PathBuf>,
    r: Option<Vec<PathBuf>>,
    sc_type: Type,
    sc_mode: raw::Mode,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    let mut ft_process = None;
    let mut r_process = None;

    let mut ft_result = None;
    let mut r_result = None;

    let mut ft_failed = false;
    let mut r_failed = false;

    let ft_converted = match sc_mode {
        raw::Mode::Groups => match sc_type {
            Type::Weekly => raw_last.clone().ft_weekly_is_some().await,
            Type::Daily => raw_last.clone().ft_daily_is_some().await,
        },
        raw::Mode::Teachers => match sc_type {
            Type::Weekly => raw_last.clone().tchr_ft_weekly_is_some().await,
            Type::Daily => raw_last.clone().tchr_ft_daily_is_some().await,
        },
    };
    let r_converted = match sc_mode {
        raw::Mode::Groups => raw_last.clone().r_weekly_is_some().await,
        raw::Mode::Teachers => raw_last.clone().tchr_r_weekly_is_some().await,
    };

    if !r_converted && r.is_some() {
        match sc_mode {
            raw::Mode::Groups => {
                r_process = Some(poll_remote(
                    r.as_ref().unwrap().clone(),
                    sc_mode.clone(),
                    raw_last.clone()
                ));
            },
            raw::Mode::Teachers => {
                r_result = Some(await_remote(
                    r.as_ref().unwrap().clone(),
                    sc_mode.clone(),
                    raw_last.clone()
                ).await);

                if r_result.as_ref().unwrap().is_err() {
                    warn!("remote {} parsing error: {:?}", sc_type, r_result.as_ref().unwrap());
                }
            }
        }
    }

    if !ft_converted && ft.is_some() {
        ft_process = Some(poll_fulltime(
            ft.as_ref().unwrap().clone(),
            match sc_mode {
                raw::Mode::Groups => {
                    match sc_type {
                        Type::Weekly => raw::Type::FtWeekly,
                        Type::Daily => raw::Type::FtDaily
                    }
                },
                raw::Mode::Teachers => {
                    match sc_type {
                        Type::Weekly => raw::Type::TchrFtWeekly,
                        Type::Daily => raw::Type::TchrFtDaily
                    }
                },
            },
            raw_last.clone(),
            if matches!(sc_mode, raw::Mode::Teachers) {
                let tchr_r_weekly = raw_last.tchr_r_weekly.read().await;
                tchr_r_weekly.as_ref().map(
                    |sch| sch.num_time_mappings.clone()
                ).flatten()
            } else {
                None
            }
        ));
    }

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

    if !ft_converted && (ft_result.is_none() || ft_result.as_ref().unwrap().is_err()) {
        ft_failed = true;
    }

    if !r_converted && (r_result.is_none() || r_result.as_ref().unwrap().is_err()) {
        r_failed = true;
    }

    match handle_fail(
        ft_failed,
        r_failed,
        sc_type.clone(),
        sc_mode.clone(),
        last.clone(),
        raw_last.clone()
    ).await {
        ControlFlow::Break(_) => return Ok(()),
        ControlFlow::Continue(_) => ()
    }

    match sc_mode {
        raw::Mode::Groups => {
            let arc_ft_page = match sc_type {
                Type::Weekly => raw_last.ft_weekly.read().await.clone(),
                Type::Daily => raw_last.ft_daily.read().await.clone(),
            };
            let ft_page = arc_ft_page.map(|arc_ft_page| (*arc_ft_page).clone());
            let r_page = {
                let arc_page = raw_last.r_weekly.read().await.clone();
                arc_page.map(|arc_page| (*arc_page).clone())
            };
            
            if ft_page.is_some() && r_page.is_none() {
                let ft_page = ft_page.unwrap();
                match sc_type {
                    Type::Daily => last.set_daily(ft_page).await,
                    Type::Weekly => last.set_weekly(ft_page).await,
                };
            } else if ft_page.is_none() && r_page.is_some() {
                let r_page = r_page.unwrap();
                match sc_type {
                    Type::Daily => (),
                    Type::Weekly => last.set_weekly(r_page).await,
                };
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
        },
        raw::Mode::Teachers => {
            let arc_ft_page = match sc_type {
                Type::Weekly => raw_last.tchr_ft_weekly.read().await.clone(),
                Type::Daily => raw_last.tchr_ft_daily.read().await.clone(),
            };
            let ft_page = arc_ft_page.map(|arc_ft_page| (*arc_ft_page).clone());
            let r_page = {
                let arc_page = raw_last.tchr_r_weekly.read().await.clone();
                arc_page.map(|arc_page| (*arc_page).clone())
            };
            
            if ft_page.is_some() && r_page.is_none() {
                let ft_page = ft_page.unwrap();
                match sc_type {
                    Type::Daily => last.set_tchr_daily(ft_page).await,
                    Type::Weekly => last.set_tchr_weekly(ft_page).await,
                };
            } else if ft_page.is_none() && r_page.is_some() {
                let r_page = r_page.unwrap();
                match sc_type {
                    Type::Daily => (),
                    Type::Weekly => last.set_tchr_weekly(r_page).await,
                };
            } else if ft_page.is_some() && r_page.is_some() {
                let mut ft_page = ft_page.unwrap();
                let mut r_page = r_page.unwrap();
        
                match sc_type {
                    Type::Weekly => {
                        if let Err(different_weeks) = merge::weekly::tchr_page(
                            &mut ft_page, 
                            &mut r_page
                        ).await {
                            ft_page = match different_weeks.latest {
                                raw::Type::TchrFtWeekly => ft_page,
                                raw::Type::TchrRWeekly => r_page,
                                _ => unreachable!()
                            }
                        }
                    
                        last.set_tchr_weekly(ft_page).await;
                    },
                    Type::Daily => {
                        if let Err(ft_not_in_r_range) = merge::daily::tchr_page(
                            &mut ft_page,
                            &mut r_page
                        ).await {
                            ft_page = match ft_not_in_r_range.latest {
                                raw::Type::TchrFtDaily => { ft_page },
                                raw::Type::TchrRWeekly => {
                                    for teacher in r_page.teachers.iter_mut() {
                                        // remain only first day of the week
                                        teacher.remove_days_except(r_page.date.start());
                                    }
                                    r_page.teachers.retain(|tchr| !tchr.days.is_empty());
                                    r_page
                                },
                                _ => unreachable!()
                            }
                        }
                    
                        last.set_tchr_daily(ft_page).await
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn weekly(
    ft_weekly: Option<PathBuf>,
    r_weekly: Option<Vec<PathBuf>>,
    sc_mode: raw::Mode,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    generic(ft_weekly, r_weekly, Type::Weekly, sc_mode, last, raw_last).await
}

pub async fn daily(
    ft_daily: Option<PathBuf>,
    r_weekly: Option<Vec<PathBuf>>,
    sc_mode: raw::Mode,
    last: Arc<Last>,
    raw_last: Arc<raw::Last>,
) -> SyncResult<()> {
    generic(ft_daily, r_weekly, Type::Daily, sc_mode, last, raw_last).await
}
