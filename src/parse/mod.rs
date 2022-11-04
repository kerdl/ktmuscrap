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
    SyncResult, 
    api::error::{
        self as api_err, 
        base::{
            ToApiError, 
            ApiError
        }
    }, 
    data::schedule::{self, raw}
};
use super::merge;


/// ## Pre-check if everything necessary is set
/// 
/// - `sc_type=Weekly` requires `ft_weekly` and `r_weekly`
/// - `sc_type=Daily` requires `ft_daily` and `r_weekly`
async fn pre_check(sc_type: schedule::Type) -> Result<(), ApiError> {
    
    let ft_weekly = crate::RAW_SCHEDULE.ft_weekly.read().await;
    let ft_daily = crate::RAW_SCHEDULE.ft_daily.read().await;
    let r_weekly = crate::RAW_SCHEDULE.r_weekly.read().await;

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
    ft_weekly: Arc<RwLock<raw::Zip>>,
    r_weekly: Arc<RwLock<raw::Zip>>
) -> SyncResult<schedule::Page> {
    pre_check(schedule::Type::Weekly).await?;

    let mut ft_weekly_page = fulltime::parse_ft_weekly(ft_weekly).await?;
    let mut r_weekly_page = remote::parse(r_weekly).await?;

    merge::weekly::page(
        &mut ft_weekly_page, 
        &mut r_weekly_page
    )?;

    Ok(ft_weekly_page)
}

pub async fn daily(
    ft_daily: Arc<RwLock<raw::Zip>>,
    r_weekly: Arc<RwLock<raw::Zip>>
) -> SyncResult<schedule::Page> {
    pre_check(schedule::Type::Daily).await?;

    let mut ft_daily_page = fulltime::parse_ft_daily(ft_daily).await?;
    let mut r_weekly_page = remote::parse(r_weekly).await?;

    merge::daily::page(
        &mut ft_daily_page, 
        &mut r_weekly_page
    )?;

    Ok(ft_daily_page)
}