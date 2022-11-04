use chrono::NaiveDate;

use crate::{data::schedule::{Page, Group, Day, raw}, SyncResult};
use super::error;


/// # Moves data from `r_day` to `ft_day`
pub async fn day(
    ft_day: &mut Day,
    r_day: &mut Day
) {
    ft_day.subjects.append(&mut r_day.subjects);
}

/// # Moves data from `r_group` to `ft_group`
pub async fn group(
    ft_date: &NaiveDate,
    ft_group: &mut Group,
    r_group: &mut Group,
) {
    let r_days = {
        let mut v = vec![];
        v.append(&mut r_group.days);
        v
    };

    for mut r_day in r_days.into_iter() {

        if &r_day.date != ft_date {
            continue;
        }

        if let Some(ft_day) = ft_group.days.iter_mut().find(
            |ft_day| &ft_day.date == ft_date
        ) {
            day(ft_day, &mut r_day).await;
        }
    }

    ft_group.days.sort();
}


/// # Moves data from `r_weekly` to `ft_daily`
pub async fn page(
    ft_daily: &mut Page, 
    r_weekly: &mut Page,
) -> SyncResult<()> {

    let ft_date = ft_daily.date.start;

    if !r_weekly.date.contains(&ft_date) {
        return Err(error::FtDateIsNotInRWeeklyRange.into())
    }

    let r_groups = {
        let mut v = vec![];
        v.append(&mut r_weekly.groups);
        v
    };

    for mut r_group in r_groups.into_iter() {

        if let Some(ft_group) = ft_daily.groups.iter_mut().find(
            |ft_group| ft_group.name == r_group.name
        ) {
            group(&ft_date, ft_group, &mut r_group).await;
        } else {
            r_group.remove_days_except(ft_date);
            ft_daily.groups.push(r_group);
        }
    }

    ft_daily.raw_types = vec![
        raw::Type::FtDaily, 
        raw::Type::RWeekly
    ];

    Ok(())
}