pub mod fulltime;
pub mod remote;

use chrono::NaiveDate;

use crate::data::schedule;
use crate::api::error::{self, base::{ApiError, ToApiError}};


/// ## Pre-check if everything necessary is set
async fn pre_check() -> Result<(), ApiError> {
    
    let ft_weekly = crate::RAW_SCHEDULE.ft_weekly.read().await;
    let ft_daily = crate::RAW_SCHEDULE.ft_daily.read().await;
    let r_weekly = crate::RAW_SCHEDULE.r_weekly.read().await;

    let no_loaded_schedules = {
        let ft_weekly_content = ft_weekly.content.read().await;
        let ft_daily_content = ft_daily.content.read().await;
        let r_weekly_content = r_weekly.content.read().await;

        ft_weekly_content.is_none()
        && ft_daily_content.is_none()
        && r_weekly_content.is_none()
    };

    if no_loaded_schedules {
        return Err(error::NO_SCHEDULES_LOADED.clone())
    }

    let unset_regexes = crate::REGEX.clone().unset_types().await;

    // if we have unset regexes
    if !unset_regexes.is_empty() {
        return Err(error::RegexesNotSet::new(unset_regexes).to_api_error())
    }

    
    Ok(())
}


pub async fn weekly() -> Result<schedule::Page, ApiError> {
    pre_check().await?;

    Ok(schedule::Page {
        raw: "".to_owned(), 
        date: NaiveDate::from_ymd(2020, 1, 1),
        groups: vec![]
    })
}

pub async fn daily() -> Result<schedule::Page, ApiError> {
    pre_check().await?;

    Ok(schedule::Page {
        raw: "".to_owned(), 
        date: NaiveDate::from_ymd(2020, 1, 1),
        groups: vec![]
    })
}