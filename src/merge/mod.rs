pub mod error;

use chrono::NaiveDate;
use error::{MergeError, NonOverlappingDates};
use std::ops::RangeInclusive;
use crate::data::schedule::{
    raw,
    attender,
    Page,
    Formation,
    Day,
    Subject,
    Attender,
    Cabinet,
};


/// # Complement two schedules
/// There is a possibility that one
/// type of schedule has data that
/// the other one doesn't have.
/// This function complements both.
/// 
/// ## For example
/// - `groups` might take cabinets
/// from `teachers`
/// - `teachers` take subject names
/// from `groups`
/// 
/// (I AM SO PROUD OF THIS CODE,
/// SMELLS AS SWEET AS INDIA)
pub fn complement<'a>(
    groups: &'a mut Page, 
    teachers: &'a mut Page,
) -> Result<(), MergeError<'a>> {
    if groups.kind != raw::Kind::Groups {
        return Err(MergeError::InvalidKind(groups));
    }
    if teachers.kind != raw::Kind::Teachers {
        return Err(MergeError::InvalidKind(teachers));
    }

    let groups_date = groups.date.start();

    if !teachers.date.contains(&groups_date) {
        let overlap_error = NonOverlappingDates {
            latest: if groups_date > teachers.date.end() {
                groups
            } else {
                teachers
            }
        };
        return Err(MergeError::NonOverlappingDates(overlap_error))
    }

    for group in groups.formations.iter_mut() {
        for group_day in group.days.iter_mut() {
            for group_subject in group_day.subjects.iter_mut() {
                for group_attender in group_subject.attenders.iter_mut() {
                    if group_attender.kind == attender::Kind::Vacancy {
                        continue
                    }

                    // find teacher mapping
                    let mut teacher = teachers.formations
                        .iter_mut()
                        .find(|tchr| tchr.name == group_attender.name);
                    
                    if teacher.is_none() {
                        let form = Formation {
                            raw: group_attender.raw.clone(),
                            recovered: true,
                            name: group_attender.name.clone(),
                            days: vec![]
                        };
                        teachers.formations.push(form);
                        teacher = teachers.formations.last_mut();
                    }

                    let teacher = teacher.unwrap();

                    // find the same day within the teacher mapping
                    let mut teacher_day = teacher.days
                        .iter_mut()
                        .find(|tchr_day| tchr_day.date == group_day.date);

                    if teacher_day.is_none() {
                        let day = Day {
                            raw: group_day.raw.clone(),
                            recovered: true,
                            date: group_day.date,
                            subjects: vec![]
                        };
                        teacher.days.push(day);
                        teacher_day = teacher.days.last_mut();
                    }
    
                    let teacher_day = teacher_day.unwrap();

                    // find the same subject within the teacher day
                    let mut teacher_subject = teacher_day.subjects
                        .iter_mut()
                        .find(|tchr_subj| tchr_subj.num == group_subject.num);

                    if teacher_subject.is_none() {
                        let subject = Subject {
                            raw: group_subject.raw.clone(),
                            recovered: true,
                            name: group_subject.name.clone(),
                            num: group_subject.num,
                            format: group_subject.format,
                            attenders: vec![]
                        };
                        teacher_day.subjects.push(subject);
                        teacher_subject = teacher_day.subjects.last_mut();
                    }

                    let teacher_subject = teacher_subject.unwrap();

                    // find the group as a teacher attender
                    let mut teacher_attender = teacher_subject.attenders
                        .iter_mut()
                        .find(|tchr_attender| tchr_attender.name == group.name);

                    if teacher_attender.is_none() {
                        let attender = Attender {
                            raw: group.raw.clone(),
                            recovered: true,
                            kind: attender::Kind::Group,
                            name: group.name.clone(),
                            cabinet: group_attender.cabinet.clone().swapped()
                        };
                        teacher_subject.attenders.push(attender);
                        teacher_attender = teacher_subject.attenders.last_mut();
                    }
    
                    let teacher_attender = teacher_attender.unwrap();

                    // group schedules have better subject naming,
                    // clone it to the teachers
                    teacher_subject.name = group_subject.name.clone();

                    teacher_attender.cabinet.opposite = group_attender
                        .cabinet
                        .primary
                        .as_ref()
                        .map(|cab| cab.clone());
                    group_attender.cabinet.opposite = teacher_attender
                        .cabinet
                        .primary
                        .as_ref()
                        .map(|cab| cab.clone());
                }
            }
        }
    }

    for teacher in teachers.formations.iter_mut() {
        for teacher_day in teacher.days.iter_mut() {
            for teacher_subject in teacher_day.subjects.iter_mut() {
                for teacher_attender in teacher_subject.attenders.iter_mut() {
                    if teacher_attender.kind == attender::Kind::Vacancy {
                        continue
                    }

                    // find group mapping
                    let mut group = groups.formations
                        .iter_mut()
                        .find(|group| group.name == teacher_attender.name);
                    
                    if group.is_none() {
                        let form = Formation {
                            raw: teacher_attender.raw.clone(),
                            recovered: true,
                            name: teacher_attender.name.clone(),
                            days: vec![]
                        };
                        groups.formations.push(form);
                        group = groups.formations.last_mut();
                    }

                    let group = group.unwrap();

                    // find the same day within the group mapping
                    let mut group_day = group.days
                        .iter_mut()
                        .find(|group_day| group_day.date == teacher_day.date);

                    if group_day.is_none() {
                        let day = Day {
                            raw: teacher_day.raw.clone(),
                            recovered: true,
                            date: teacher_day.date,
                            subjects: vec![]
                        };
                        group.days.push(day);
                        group_day = group.days.last_mut();
                    }
    
                    let group_day = group_day.unwrap();

                    // find the same subject within the group day
                    let mut group_subject = group_day.subjects
                        .iter_mut()
                        .find(|group_subj| group_subj.num == teacher_subject.num);

                    if group_subject.is_none() {
                        let subject = Subject {
                            raw: teacher_subject.raw.clone(),
                            recovered: true,
                            name: teacher_subject.name.clone(),
                            num: teacher_subject.num,
                            format: teacher_subject.format,
                            attenders: vec![]
                        };
                        group_day.subjects.push(subject);
                        group_subject = group_day.subjects.last_mut();
                    }

                    let group_subject = group_subject.unwrap();

                    // find the teacher as a group attender
                    let mut group_attender = group_subject.attenders
                        .iter_mut()
                        .find(|group_attender| group_attender.name == teacher.name);

                    if group_attender.is_none() {
                        let attender = Attender {
                            raw: teacher.raw.clone(),
                            recovered: true,
                            kind: attender::Kind::Teacher,
                            name: teacher.name.clone(),
                            cabinet: teacher_attender.cabinet.clone().swapped()
                        };
                        group_subject.attenders.push(attender);
                        group_attender = group_subject.attenders.last_mut();
                    }
    
                    let group_attender = group_attender.unwrap();

                    if group_attender.cabinet.opposite.is_none() {
                        group_attender.cabinet.opposite = teacher_attender
                            .cabinet
                            .primary
                            .as_ref()
                            .map(|cab| cab.clone());
                    }
                    if teacher_attender.cabinet.opposite.is_none() {
                        teacher_attender.cabinet.opposite = group_attender
                            .cabinet
                            .primary
                            .as_ref()
                            .map(|cab| cab.clone());
                    }
                }
            }
        }
    }

    groups.formations.iter_mut().for_each(|form| {
        form.days.sort_by(|a, b| a.date.cmp(&b.date));
        form.days.iter_mut().for_each(|day| {
            day.subjects.sort_by(|a, b| a.num.cmp(&b.num));
        })
    });

    teachers.formations.iter_mut().for_each(|form| {
        form.days.sort_by(|a, b| a.date.cmp(&b.date));
        form.days.iter_mut().for_each(|day| {
            day.subjects.sort_by(|a, b| a.num.cmp(&b.num));
        })
    });

    Ok(())
}

fn combine_cabinets(dst: &mut Cabinet, src: Cabinet) {
    if let (Some(dst_primary), Some(src_primary)) = (&mut dst.primary, src.primary) {
        if *dst_primary != src_primary {
            dst_primary.push_str(", ");
            dst_primary.push_str(&src_primary);
        }
    }
    if let (Some(dst_opposite), Some(src_opposite)) = (&mut dst.opposite, src.opposite) {
        if *dst_opposite != src_opposite {
            dst_opposite.push_str(", ");
            dst_opposite.push_str(&src_opposite);
        }
    }
}

fn combine_attenders(dst: &mut Vec<Attender>, src: Vec<Attender>) {
    for src_att in src {
        if let Some(existing_att) = dst
            .iter_mut()
            .find(|dst_att|
                dst_att.kind == src_att.kind &&
                dst_att.name == src_att.name 
            )
        {
            combine_cabinets(
                &mut existing_att.cabinet,
                src_att.cabinet
            )
        } else {
            dst.push(src_att)
        }
    }
}

fn combine_subjects(dst: &mut Vec<Subject>, src: Vec<Subject>) {
    for src_subject in src {
        if let Some(existing_subject) = dst
            .iter_mut()
            .find(|dst_subject|
                dst_subject.num == src_subject.num &&
                dst_subject.format == src_subject.format
            )
        {
            combine_attenders(
                &mut existing_subject.attenders,
                src_subject.attenders
            )
        } else {
            dst.push(src_subject)
        }
    }
}

fn combine_days(dst: &mut Vec<Day>, src: Vec<Day>) {
    for src_day in src {
        if let Some(existing_day) = dst
            .iter_mut()
            .find(|dst_day| dst_day.date == src_day.date)
        {
            combine_subjects(
                &mut existing_day.subjects,
                src_day.subjects
            )
        } else {
            dst.push(src_day)
        }
    }
}

/// # Combine multiple pages
pub fn combine(
    pages: Vec<Page>,
    date: RangeInclusive<NaiveDate>,
    kind: raw::Kind
) -> Page {
    let mut new_page = Page {
        kind,
        date,
        formations: vec![]
    };

    for page in pages {
        for formation in page.formations {
            if let Some(existing_formation) = new_page.formations
                .iter_mut()
                .find(|existing| existing.name == formation.name)
            {
                combine_days(
                    &mut existing_formation.days,
                    formation.days
                );
            } else {
                new_page.formations.push(formation);
            }
        }
    }

    new_page
}