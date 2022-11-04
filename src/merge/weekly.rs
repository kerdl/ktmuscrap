use log::info;
use chrono::{Datelike, Weekday};

use crate::{
    data::schedule::{
        Page,
        Group,
    },
    SyncResult
};
use super::error;

pub fn group(
    ft_group: &mut Group,
    r_group: &mut Group,
) {
    
}

pub fn page(
    ft_weekly: &mut Page, 
    r_weekly: &mut Page,
) -> SyncResult<()> {

    let ft_week = {
        let week = ft_weekly.date.start.week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    let r_week = {
        let week = r_weekly.date.start.week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    if ft_week != r_week {
        return Err(error::DifferentWeeks.into())
    }

    let r_groups = {
        let mut v = vec![];
        v.append(&mut r_weekly.groups);
        v
    };

    for mut r_group in r_groups.into_iter() {
        if let Some(ft_group) = ft_weekly.groups.iter_mut().find(
            |ft_group| ft_group.name == r_group.name
        ) {
            group(ft_group, &mut r_group);
        } else {
            ft_weekly.groups.push(r_group);
        }
    }

    Ok(())
}