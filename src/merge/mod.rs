pub mod error;

use error::{MergeError, NonOverlappingDates};
use crate::data::schedule::{
    raw,
    attender,
    Attender,
    Page
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
pub async fn page<'a>(
    groups: &'a mut Page, 
    teachers: &'a mut Page,
) -> Result<(), MergeError<'a>> {
    if groups.kind != Some(raw::Kind::Groups) {
        return Err(MergeError::InvalidKind(groups));
    }
    if teachers.kind != Some(raw::Kind::Teachers) {
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

    for group in groups.mappings.iter_mut() {
        for group_day in group.days.iter_mut() {
            for group_subject in group_day.subjects.iter_mut() {
                for group_attender in group_subject.attenders.iter_mut() {
                    // find teacher mapping
                    let Some(teacher) = teachers.mappings.iter_mut().find(
                        |tchr|
                        tchr.name == group_attender.name
                    ) else { continue };

                    // find the same day within the teacher mapping
                    let Some(teacher_day) = teacher.days.iter_mut().find(
                        |tchr_day|
                        tchr_day.date == group_day.date
                    ) else { continue };

                    // find the same subject within the teacher day
                    let Some(teacher_subject) = teacher_day.subjects.iter_mut().find(
                        |tchr_subj|
                        tchr_subj.num == group_subject.num
                    ) else { continue };

                    // find the group as a teacher attender
                    let teacher_attender = teacher_subject.attenders.iter_mut().find(
                        |tchr_attender|
                        tchr_attender.name == group_attender.name
                    );

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
                            cabinet: {
                                let mut cab = group_attender.cabinet.clone();
                                cab.swap();
                                cab
                            }
                        };
                        teacher_subject.attenders.push(group_as_attender);
                    } else {
                        let teacher_attender = teacher_attender.unwrap();

                        teacher_attender.cabinet.opposite = group_attender.cabinet.primary.as_ref().map(
                            |cab| cab.clone()
                        );
                        group_attender.cabinet.opposite = teacher_attender.cabinet.primary.as_ref().map(
                            |cab| cab.clone()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
