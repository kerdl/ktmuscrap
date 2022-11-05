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

use tokio::sync::RwLock;
use std::sync::Arc;

use crate::{
    RAW_SCHEDULE,
    LAST_SCHEDULE,
    SyncResult, 
    api::error::{
        self as api_err, 
        base::{
            ToApiError, 
            ApiError
        }
    }, 
    data::schedule::{self, raw}, perf
};
use super::merge;


/// ## Pre-check if everything necessary is set
/// 
/// - `sc_type=Weekly` requires `ft_weekly` and `r_weekly`
/// - `sc_type=Daily` requires `ft_daily` and `r_weekly`
async fn pre_check(sc_type: schedule::Type) -> Result<(), ApiError> {
    
    let ft_weekly = crate::RAW_SCHEDULE.get().unwrap().ft_weekly.zip.read().await;
    let ft_daily = crate::RAW_SCHEDULE.get().unwrap().ft_daily.zip.read().await;
    let r_weekly = crate::RAW_SCHEDULE.get().unwrap().r_weekly.zip.read().await;

    let has_weekly_schedules = {
        ft_weekly.path().exists()
        && r_weekly.path().exists()
    };
    let has_daily_schedules = {
        ft_daily.path().exists()
        && r_weekly.path().exists()
    };


    if sc_type == schedule::Type::Weekly && !has_weekly_schedules {
        return Err(api_err::NoWeeklySchedulesLoaded::new().to_api_error())
    }

    if sc_type == schedule::Type::Daily && !has_daily_schedules {
        return Err(api_err::NoDailySchedulesLoaded::new().to_api_error())
    }


    Ok(())
}


pub async fn weekly(
    ft_weekly: Arc<raw::Schedule>,
    r_weekly: Arc<raw::Schedule>
) -> SyncResult<()> {

    pre_check(schedule::Type::Weekly).await?;


    if LAST_SCHEDULE.get().unwrap().weekly.read().await.is_some() {
        return Ok(())
    }

    let mut parsing_processes = vec![];

    if ft_weekly.parsed.read().await.is_none() {

        let process = tokio::spawn(async move {
            perf!(fulltime::parse_ft_weekly(ft_weekly).await?);

            Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
        });

        parsing_processes.push(process);
    }
    
    if r_weekly.parsed.read().await.is_none() {

        let process = tokio::spawn(async move {
            perf!(remote::parse(r_weekly).await?);

            Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
        });

        parsing_processes.push(process);
    }

    for process in parsing_processes {
        process.await??;
    }

    // yes, actually clone large
    // large `Page` struct
    let mut ft_weekly_page = {
        let arc_page = {
            RAW_SCHEDULE.get().unwrap().ft_weekly.parsed
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };
    let mut r_weekly_page = {
        let arc_page = {
            RAW_SCHEDULE.get().unwrap().r_weekly.parsed
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };

    merge::weekly::page(
        &mut ft_weekly_page, 
        &mut r_weekly_page
    ).await?;

    *LAST_SCHEDULE.get().unwrap().weekly.write().await = Some(Arc::new(ft_weekly_page));
    LAST_SCHEDULE.get().unwrap().save().await?;

    Ok(())
}

pub async fn daily(
    ft_daily: Arc<raw::Schedule>,
    r_weekly: Arc<raw::Schedule>
) -> SyncResult<()> {

    pre_check(schedule::Type::Daily).await?;


    if LAST_SCHEDULE.get().unwrap().daily.read().await.is_some() {
        return Ok(())
    }

    let mut parsing_processes = vec![];

    if ft_daily.parsed.read().await.is_none() {
    
        let process = tokio::spawn(async move {
            perf!(fulltime::parse_ft_daily(ft_daily).await?);

            Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
        });

        parsing_processes.push(process);
    }
    
    if r_weekly.parsed.read().await.is_none() {

        let process = tokio::spawn(async move {
            perf!(remote::parse(r_weekly).await?);

            Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
        });

        parsing_processes.push(process);
    }

    for process in parsing_processes {
        process.await??;
    }


    // yes, actually clone large
    // large `Page` struct
    let mut ft_daily_page = {
        let arc_page = {
            RAW_SCHEDULE.get().unwrap().ft_daily.parsed
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };
    let mut r_weekly_page = {
        let arc_page = {
            RAW_SCHEDULE.get().unwrap().r_weekly.parsed
            .read().await.clone().unwrap()
        };
        (*arc_page).clone()
    };

    merge::daily::page(
        &mut ft_daily_page, 
        &mut r_weekly_page
    ).await?;

    *LAST_SCHEDULE.get().unwrap().daily.write().await = Some(Arc::new(ft_daily_page));
    LAST_SCHEDULE.get().unwrap().save().await?;

    Ok(())
}