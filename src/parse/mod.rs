pub mod fulltime;
pub mod remote;
pub mod node;
pub mod date;
pub mod time;
pub mod num;
pub mod group;
pub mod error;

use chrono::NaiveDate;

use crate::api::error::base::ToApiError;
use crate::data::schedule;
use crate::api::error::{self as api_err, base::ApiError};


/// ## Pre-check if everything necessary is set
/// 
/// - `sc_type=Weekly` requires `ft_weekly` and `r_weekly`
/// - `sc_type=Daily` requires `ft_daily` and `r_weekly`
async fn pre_check(sc_type: schedule::Type) -> Result<(), ApiError> {
    
    let ft_weekly = crate::RAW_SCHEDULE.ft_weekly.read().await;
    let ft_daily = crate::RAW_SCHEDULE.ft_daily.read().await;
    let r_weekly = crate::RAW_SCHEDULE.r_weekly.read().await;

    let ft_weekly_content = ft_weekly.content.read().await;
    let ft_daily_content = ft_daily.content.read().await;
    let r_weekly_content = r_weekly.content.read().await;


    let weekly_schedules_loaded = {
        ft_weekly_content.is_some() 
        && r_weekly_content.is_some()
    };
    let daily_schedules_loaded = {
        ft_daily_content.is_some() 
        && r_weekly_content.is_some()
    };


    if sc_type == schedule::Type::Weekly && !weekly_schedules_loaded {
        return Err(api_err::NoWeeklySchedulesLoaded::new().to_api_error())
    }

    if sc_type == schedule::Type::Daily && !daily_schedules_loaded {
        return Err(api_err::NoDailySchedulesLoaded::new().to_api_error())
    }


    Ok(())
}


pub async fn weekly() -> Result<schedule::Page, ApiError> {
    pre_check(schedule::Type::Weekly).await?;

    Ok(schedule::Page {
        raw: "".to_owned(), 
        date: NaiveDate::from_ymd(2020, 1, 1),
        groups: vec![]
    })
}

pub async fn daily() -> Result<schedule::Page, ApiError> {
    pre_check(schedule::Type::Daily).await?;

    Ok(schedule::Page {
        raw: "".to_owned(), 
        date: NaiveDate::from_ymd(2020, 1, 1),
        groups: vec![]
    })
}