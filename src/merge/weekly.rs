use log::info;
use chrono::{Datelike, Weekday};

use crate::{data::schedule::Page, SyncResult};
use super::error;


pub async fn merge(
    ft_weekly: Page, 
    r_weekly: Page,
) -> SyncResult<Page> {
    let ft_week = {
        let week = ft_weekly.date.start.week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    let r_week = {
        let week = r_weekly.date.start.week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    info!("{:?} {:?}", ft_week, r_week);

    if ft_week != r_week {
        return Err(error::DifferentWeeks.into())
    }


    todo!()
}