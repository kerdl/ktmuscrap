use chrono::Weekday;
use itertools::Itertools;

use crate::{
    data::schedule::{
        raw,
        Page,
        Group,
        Day,
        TchrPage,
        TchrTeacher,
        TchrDay
    },
    SyncResult
};
use super::error;


// groups

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
) -> Result<(), error::DifferentWeeks> {
    let ft_week = {
        let week = ft_weekly.date.start().week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    let r_week = {
        let week = r_weekly.date.start().week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    if ft_week != r_week {
        return Err(error::DifferentWeeks {
            latest: if ft_week.start > r_week.start {
                raw::Type::FtWeekly
            } else {
                raw::Type::RWeekly
            }
        }.into())
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

// teachers

pub async fn tchr_day(
    ft_day: &mut TchrDay,
    r_day: &mut TchrDay,
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

pub async fn tchr_teacher(
    ft_teacher: &mut TchrTeacher,
    r_group: &mut TchrTeacher,
) {
    let r_days = {
        let mut v = vec![];
        v.append(&mut r_group.days);
        v
    };

    for mut r_day in r_days.into_iter() {
        if let Some(ft_day) = ft_teacher.days.iter_mut().find(
            |day| day.date == r_day.date
        ) {
            tchr_day(ft_day, &mut r_day).await;
        } else {
            ft_teacher.days.push(r_day);
        }
    }

    ft_teacher.days.sort_by(
        |day_a, day_b| day_a.weekday.cmp(&day_b.weekday)
    );
}

pub async fn tchr_page(
    ft_weekly: &mut TchrPage, 
    r_weekly: &mut TchrPage,
) -> Result<(), error::DifferentWeeks> {
    let ft_week = {
        let week = ft_weekly.date.start().week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    let r_week = {
        let week = r_weekly.date.start().week(Weekday::Mon);
        week.first_day()..week.last_day()
    };

    if ft_week != r_week {
        return Err(error::DifferentWeeks {
            latest: if ft_week.start > r_week.start {
                raw::Type::FtWeekly
            } else {
                raw::Type::RWeekly
            }
        }.into())
    }

    let r_teachers = {
        let mut v = vec![];
        v.append(&mut r_weekly.teachers);
        v
    };

    for mut r_teacher in r_teachers.into_iter() {
        if let Some(ft_teacher) = ft_weekly.teachers.iter_mut().find(
            |ft_group| ft_group.name == r_teacher.name
        ) {
            tchr_teacher(ft_teacher, &mut r_teacher).await;
        } else {
            ft_weekly.teachers.push(r_teacher);
        }
    }

    ft_weekly.raw_types = vec![
        raw::Type::FtWeekly, 
        raw::Type::RWeekly
    ];

    Ok(())
}

pub async fn tchr_r_page(
    pages: &mut [TchrPage]
) -> Result<Option<TchrPage>, error::DifferentWeeks> {
    match pages.len() {
        0 => return Ok(None),
        1 => return Ok(Some(pages[0].clone())),
        _ => {}
    }

    let mut page = pages[0].clone();

    let weeks = pages.iter().map(|page| {
        let week = page.date.start().week(Weekday::Mon);
        week.first_day()..week.last_day()
    }).collect_vec();

    if !weeks.iter().all(|week| week == weeks.first().unwrap()) {
        return Err(error::DifferentWeeks {
            latest: raw::Type::RWeekly
        }.into())
    }

    let r_teachers = {
        let mut v = vec![];
        for page in pages.iter_mut() {
            v.append(&mut page.teachers);
        }
        v
    };

    for mut teacher in r_teachers.into_iter() {
        if let Some(existing_teacher) = page.teachers.iter_mut().find(
            |tchr| tchr.name == teacher.name
        ) {
            tchr_teacher(existing_teacher, &mut teacher).await;
        } else {
            page.teachers.push(teacher);
        }
    }

    Ok(Some(page))
}