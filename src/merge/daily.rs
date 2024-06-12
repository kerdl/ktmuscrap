use chrono::NaiveDate;

use crate::{data::schedule::{
    Page,
    Group,
    Day,
    TchrPage,
    TchrTeacher,
    TchrDay,
    raw
}, SyncResult};
use super::error;


// groups

/// # Moves data from `r_day` to `ft_day`
pub async fn day(
    ft_day: &mut Day,
    r_day: &mut Day
) {
    ft_day.subjects.append(&mut r_day.subjects);

    ft_day.subjects.sort_by(
        |subj_a, subj_b| subj_a.num.cmp(&subj_b.num)
    );
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

    ft_group.days.sort_by(
        |day_a, day_b| day_a.weekday.cmp(&day_b.weekday)
    );
}


/// # Moves data from `r_weekly` to `ft_daily`
pub async fn page(
    ft_daily: &mut Page, 
    r_weekly: &mut Page,
) -> Result<(), error::FtDateIsNotInRWeeklyRange> {
    let ft_date = ft_daily.date.start();

    if !r_weekly.date.contains(&ft_date) {
        return Err(error::FtDateIsNotInRWeeklyRange {
            latest: if ft_date > r_weekly.date.end() {
                raw::Type::FtDaily
            } else {
                raw::Type::RWeekly
            }
        })
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
            group(ft_date, ft_group, &mut r_group).await;
        } else {
            r_group.remove_days_except(ft_date);

            if r_group.days.is_empty() {
                continue
            }

            ft_daily.groups.push(r_group);
        }
    }

    ft_daily.raw_types = vec![
        raw::Type::FtDaily, 
        raw::Type::RWeekly
    ];

    Ok(())
}

// teachers

/// # Moves data from `r_day` to `ft_day`
pub async fn tchr_day(
    ft_day: &mut TchrDay,
    r_day: &mut TchrDay
) {
    ft_day.subjects.append(&mut r_day.subjects);

    ft_day.subjects.sort_by(
        |subj_a, subj_b| subj_a.num.cmp(&subj_b.num)
    );
}

pub async fn tchr_ft_day(
    daily_day: &mut TchrDay,
    weekly_day: &TchrDay
) {
    for daily_subject in daily_day.subjects.iter_mut() {
        if daily_subject.name.trim().is_empty() {
            let Some(weekly_subject) = weekly_day.subjects.iter().find(
                |subj| subj.num == daily_subject.num &&
                subj.format == daily_subject.format
            ) else {
                continue
            };

            daily_subject.name = weekly_subject.name.clone();
        }
    }
}

/// # Moves data from `r_teacher` to `ft_teacher`
pub async fn tchr_teacher(
    ft_date: &NaiveDate,
    ft_teacher: &mut TchrTeacher,
    r_teacher: &mut TchrTeacher,
) {
    let r_days = {
        let mut v = vec![];
        v.append(&mut r_teacher.days);
        v
    };

    for mut r_day in r_days.into_iter() {
        if &r_day.date != ft_date {
            continue;
        }

        if let Some(ft_day) = ft_teacher.days.iter_mut().find(
            |ft_day| &ft_day.date == ft_date
        ) {
            tchr_day(ft_day, &mut r_day).await;
        }
    }

    ft_teacher.days.sort_by(
        |day_a, day_b| day_a.weekday.cmp(&day_b.weekday)
    );
}

pub async fn tchr_ft_teacher(
    daily_date: &NaiveDate,
    daily_teacher: &mut TchrTeacher,
    weekly_teacher: TchrTeacher,
) {
    for mut daily_day in daily_teacher.days.iter_mut() {
        if let Some(weekly_day) = weekly_teacher.days.iter().find(
            |weekly_day| &weekly_day.date == daily_date
        ) {
            tchr_ft_day(&mut daily_day, &weekly_day).await;
        }
    }

    daily_teacher.days.sort_by(
        |day_a, day_b| day_a.weekday.cmp(&day_b.weekday)
    );
}


/// # Moves data from `r_weekly` to `ft_daily`
pub async fn tchr_page(
    ft_daily: &mut TchrPage, 
    r_weekly: &mut TchrPage,
) -> Result<(), error::FtDateIsNotInRWeeklyRange> {
    let ft_date = ft_daily.date.start();

    if !r_weekly.date.contains(&ft_date) {
        return Err(error::FtDateIsNotInRWeeklyRange {
            latest: if ft_date > r_weekly.date.end() {
                raw::Type::TchrFtDaily
            } else {
                raw::Type::TchrRWeekly
            }
        })
    }

    let r_teachers = {
        let mut v = vec![];
        v.append(&mut r_weekly.teachers);
        v
    };

    for mut r_teacher in r_teachers.into_iter() {
        if let Some(ft_teacher) = ft_daily.teachers.iter_mut().find(
            |ft_teacher| ft_teacher.name == r_teacher.name
        ) {
            tchr_teacher(ft_date, ft_teacher, &mut r_teacher).await;
        } else {
            r_teacher.remove_days_except(ft_date);

            if r_teacher.days.is_empty() {
                continue
            }

            ft_daily.teachers.push(r_teacher);
        }
    }

    ft_daily.raw_types = vec![
        raw::Type::TchrFtDaily, 
        raw::Type::TchrRWeekly
    ];

    Ok(())
}

pub async fn tchr_ft_page(
    daily: &mut TchrPage, 
    weekly: TchrPage,
) -> Result<(), error::FtDateIsNotInRWeeklyRange> {
    let daily_date = daily.date.start();

    if !weekly.date.contains(&daily_date) {
        return Err(error::FtDateIsNotInRWeeklyRange {
            latest: if daily_date > weekly.date.end() {
                raw::Type::TchrFtDaily
            } else {
                raw::Type::TchrFtWeekly
            }
        })
    }

    for mut daily_teacher in daily.teachers.iter_mut() {
        if let Some(weekly_teacher) = weekly.teachers.iter().find(
            |weekly_teacher| weekly_teacher.name == daily_teacher.name
        ) {
            tchr_ft_teacher(daily_date, &mut daily_teacher, weekly_teacher.clone()).await;
        }
    }

    daily.raw_types = vec![
        raw::Type::TchrFtDaily, 
        raw::Type::TchrFtWeekly
    ];

    Ok(())
}