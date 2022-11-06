use log::info;
use chrono::Weekday;

use crate::{
    data::schedule::{
        raw,
        Page,
        Group,
        Day,
    },
    SyncResult
};
use super::error;


pub async fn day(
    ft_day: &mut Day,
    r_day: &mut Day,
) {
    let r_subjects = {
        let mut v = vec![];
        v.append(&mut r_day.subjects);
        v
    };

    for r_subject in r_subjects.into_iter() {
        ft_day.subjects.push(r_subject);
    }

    ft_day.subjects.sort_by(
        |subj_a, subj_b| subj_a.num.cmp(&subj_b.num)
    );
}

pub async fn group(
    ft_group: &mut Group,
    r_group: &mut Group,
) {
    let r_days = {
        let mut v = vec![];
        v.append(&mut r_group.days);
        v
    };

    for mut r_day in r_days.into_iter() {
        if let Some(ft_day) = ft_group.days.iter_mut().find(
            |day| day.date == r_day.date
        ) {
            day(ft_day, &mut r_day).await;
        } else {
            ft_group.days.push(r_day);
        }
    }

    ft_group.days.sort_by(
        |day_a, day_b| day_a.weekday.cmp(&day_b.weekday)
    );
}

pub async fn page(
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
            group(ft_group, &mut r_group).await;
        } else {
            ft_weekly.groups.push(r_group);
        }
    }

    ft_weekly.raw_types = vec![
        raw::Type::FtWeekly, 
        raw::Type::RWeekly
    ];

    Ok(())
}