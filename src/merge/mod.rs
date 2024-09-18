pub mod error;

use chrono::NaiveDate;
use error::{MergeError, NonOverlappingDates};
use std::ops::RangeInclusive;
use crate::data::schedule::{
    raw,
    attender,
    Page,
    Day,
    Subject,
    Attender,
    Cabinet
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
                    // find teacher mapping
                    let Some(teacher) = teachers.formations
                        .iter_mut()
                        .find(|tchr| tchr.name == group_attender.name)
                        else { continue };

                    // find the same day within the teacher mapping
                    let Some(teacher_day) = teacher.days
                        .iter_mut()
                        .find(|tchr_day| tchr_day.date == group_day.date)
                        else { continue };

                    // find the same subject within the teacher day
                    let Some(teacher_subject) = teacher_day.subjects
                        .iter_mut()
                        .find(|tchr_subj| tchr_subj.num == group_subject.num)
                        else { continue };

                    // find the group as a teacher attender
                    let teacher_attender = teacher_subject.attenders
                        .iter_mut()
                        .find(|tchr_attender| tchr_attender.name == group_attender.name);

                    // group schedules have better subject naming,
                    // clone it to the teachers
                    teacher_subject.name = group_subject.name.clone();

                    // if the teacher doesn't have this group as an attender
                    // (probably won't happen),
                    // add it manually 
                    if teacher_attender.is_none() {
                        let group_as_attender = Attender {
                            raw: group.raw.clone(),
                            kind: attender::Kind::Group,
                            name: group.name.clone(),
                            cabinet: group_attender.cabinet.clone().swapped()
                        };
                        teacher_subject.attenders.push(group_as_attender);
                    } else {
                        let teacher_attender = teacher_attender.unwrap();

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
    }

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