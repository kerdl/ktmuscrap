use derive_new::new;
use itertools::Itertools;
use std::collections::HashSet;

use crate::data::schedule::{
    raw::{
        self,
        remote::table::{
            SubjectMapping,
            TchrSubjectMapping,
            WeekdayDate
        },
    },
    Day,
    Format,
    Group,
    Page,
    Subject,
    TchrPage,
    TchrTeacher,
    TchrDay,
    TchrSubject,
    Type
};
use super::super::{
    teacher,
    subject,
    group
};


/// # 3rd, final step of parsing remote schedule
#[derive(new, Debug, Clone)]
pub struct Parser {
    schema: Vec<Vec<SubjectMapping>>,
    weekday_date_row: Option<Vec<WeekdayDate>>,

    pub page: Option<Page>
}
impl Parser {
    pub fn from_schema(
        schema: Vec<Vec<SubjectMapping>>,
        weekday_date_row: Option<Vec<WeekdayDate>>
    ) -> Parser {
        Parser::new(schema, weekday_date_row, None)
    }

    pub fn page(&mut self) -> Option<&Page> {
        if self.page.is_some() {
            return Some(self.page.as_ref().unwrap())
        }

        let base_weekday: &WeekdayDate = &self.schema.get(0)?.get(0)?.weekday_date;

        let mut groups: Vec<Group> = vec![];


        for group_row in self.schema.iter() {
            let group_name = &group_row.get(0)?.group;

            let mut days: Vec<Day> = vec![];
            let mut subjects: Vec<Subject> = vec![];


            for (index, subject) in group_row.iter().enumerate() {
                let next_subject = group_row.get(index + 1);

                let mut subject_strings: Vec<String> = vec![];

                if subject::is_split(&subject.cell.text) {
                    subject_strings.extend_from_slice(
                        &subject::split(&subject.cell.text)
                    );
                } else {
                    subject_strings.push(
                        subject.cell.text.to_owned()
                    );
                }

                for subject_string in subject_strings.iter_mut() {
                    if subject_string.is_empty() { continue; }

                    let teachers = teacher::extract_from_end(subject_string);

                    let mut subject = Subject {
                        raw:      subject.cell.text.clone(),
                        num:      subject.num_time.num,
                        time:     subject.num_time.time.clone(),
                        name:     subject_string.clone(),
                        format:   Format::Remote,
                        teachers,
                        cabinet:  None,
                    };

                    if subject.is_fulltime_window() {
                        continue;
                    }

                    let existing_subjects = subjects.iter_mut().filter(
                        |subj| subj.name == subject.name &&
                        subj.num == subject.num &&
                        subj.time == subject.time
                    ).collect::<Vec<&mut Subject>>();

                    if !existing_subjects.is_empty() {
                        for existing_subject in existing_subjects {
                            existing_subject.teachers.append(&mut subject.teachers);
                        }
                    } else {
                        subjects.push(subject);
                    }
                }

                
                let is_changing_weekday = {
                    next_subject.is_some()
                    && next_subject.as_ref().unwrap().weekday_date != subject.weekday_date
                };

                let was_last = {
                    next_subject.is_none()
                };

                if (is_changing_weekday || was_last) && !subjects.is_empty() {
                    let day = Day {
                        raw:      subject.weekday_date.cell.text.clone(),
                        weekday:  subject.weekday_date.weekday.clone(),
                        date:     subject.weekday_date.date.clone(),
                        subjects: {
                            let mut subjs = vec![];
                            subjs.append(&mut subjects);
                            subjs
                        }
                    };
                    days.push(day);
                }
            }

            if days.is_empty() {
                continue
            }

            let group = Group {
                raw:  group_name.raw.to_owned(),
                name: group_name.valid.to_owned(),
                days
            };

            groups.push(group);
        }

        let page = Page {
            raw:       base_weekday.cell.text.to_owned(),
            raw_types: vec![raw::Type::RWeekly],
            sc_type:   Type::Weekly,
            date: {
                let header_row = self.schema.get(0).unwrap();

                let start = header_row.first().unwrap().weekday_date.date;
                let end = header_row.last().unwrap().weekday_date.date;

                start..=end
            },
            groups,
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }

    pub fn page_v2(&mut self) -> Option<&Page> {
        if self.page.is_some() {
            return Some(self.page.as_ref().unwrap())
        }

        let base_weekday: &WeekdayDate = &self.schema.get(0)?.get(0)?.weekday_date;

        let mut groups: Vec<Group> = vec![];


        for group_row in self.schema.iter() {
            let group_name = &group_row.get(0)?.group;

            let mut days: Vec<Day> = vec![];
            let mut subjects: Vec<Subject> = vec![];


            for (index, subject) in group_row.iter().enumerate() {
                let next_subject = group_row.get(index + 1);

                let mut subject_strings: Vec<String> = vec![];

                if subject::is_split(&subject.cell.text) {
                    subject_strings.extend_from_slice(
                        &subject::split(&subject.cell.text)
                    );
                } else {
                    subject_strings.push(
                        subject.cell.text.to_owned()
                    );
                }

                for subject_string in subject_strings.iter_mut() {
                    if subject_string.is_empty() { continue; }

                    let teachers = teacher::extract_from_end(subject_string);

                    let mut subject = Subject {
                        raw:      subject.cell.text.clone(),
                        num:      subject.num_time.num,
                        time:     subject.num_time.time.clone(),
                        name:     subject_string.clone(),
                        format:   Format::Remote,
                        teachers,
                        cabinet:  None,
                    };

                    if subject.is_fulltime_window() {
                        continue;
                    }

                    let existing_subjects = subjects.iter_mut().filter(
                        |subj| subj.name == subject.name &&
                        subj.num == subject.num &&
                        subj.time == subject.time
                    ).collect::<Vec<&mut Subject>>();

                    if !existing_subjects.is_empty() {
                        for existing_subject in existing_subjects {
                            existing_subject.teachers.append(&mut subject.teachers);
                        }
                    } else {
                        subjects.push(subject);
                    }
                }

                
                let is_changing_weekday = {
                    next_subject.is_some()
                    && next_subject.as_ref().unwrap().weekday_date != subject.weekday_date
                };

                let was_last = {
                    next_subject.is_none()
                };

                if (is_changing_weekday || was_last) && !subjects.is_empty() {
                    let day = Day {
                        raw:      subject.weekday_date.cell.text.clone(),
                        weekday:  subject.weekday_date.weekday.clone(),
                        date:     subject.weekday_date.date.clone(),
                        subjects: {
                            let mut subjs = vec![];
                            subjs.append(&mut subjects);
                            subjs
                        }
                    };
                    days.push(day);
                }
            }

            if days.is_empty() {
                continue
            }

            let group = Group {
                raw:  group_name.raw.to_owned(),
                name: group_name.valid.to_owned(),
                days
            };

            groups.push(group);
        }

        let page = Page {
            raw:       base_weekday.cell.text.to_owned(),
            raw_types: vec![raw::Type::RWeekly],
            sc_type:   Type::Weekly,
            date: {
                let wkd_row = self.weekday_date_row.as_ref().unwrap();

                let start = wkd_row.first().unwrap().date;
                let end = wkd_row.last().unwrap().date;

                start..=end
            },
            groups,
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }
}

/// # 3rd, final step of parsing remote schedule
#[derive(new, Debug, Clone)]
pub struct TchrParser {
    schema: Vec<Vec<TchrSubjectMapping>>,
    weekday_date_range: Option<Vec<WeekdayDate>>,

    pub page: Option<TchrPage>
}
impl TchrParser {
    pub fn from_schema(
        schema: Vec<Vec<TchrSubjectMapping>>,
        weekday_date_range: Option<Vec<WeekdayDate>>
    ) -> Self {
        Self::new(schema, weekday_date_range, None)
    }

    pub fn page(&mut self) -> Option<&TchrPage> {
        if self.page.is_some() {
            return Some(self.page.as_ref().unwrap())
        }

        let base_weekday: &WeekdayDate = &self.schema.get(0)?.get(0)?.weekday_date;

        let mut teachers: Vec<TchrTeacher> = vec![];

        for weekday_row in self.schema.iter() {
            for map in weekday_row.iter() {
                let teacher_name = &map.teacher;
                let teacher = match teachers.iter_mut().find(|teacher| teacher.name == teacher_name.valid) {
                    Some(teacher) => teacher,
                    None => {
                        let teacher = TchrTeacher {
                            raw: map.teacher.raw.clone(),
                            name: map.teacher.valid.clone(),
                            days: vec![],
                        };
                        teachers.push(teacher);
                        teachers.last_mut().unwrap()
                    }
                };

                let day = match teacher.days.iter_mut().find(|day| day.weekday == map.weekday_date.weekday) {
                    Some(day) => day,
                    None => {
                        let day = TchrDay {
                            raw: map.weekday_date.cell.text.clone(),
                            weekday: map.weekday_date.weekday.clone(),
                            date: map.weekday_date.date,
                            subjects: vec![],
                        };
                        teacher.days.push(day);
                        teacher.days.last_mut().unwrap()
                    }
                };

                let (groups, name) = map.cell.text.split_once(
                    "*&^%$#@!FUCKING_SEPARATOR!@#$%^&*"
                ).unwrap();
                let groups = group::parse_multiple(groups);

                let mut subject_created = false;
                let subject = match day.subjects.iter_mut().find(
                    |subject|
                        subject.name == name &&
                        subject.num == map.num_time.num &&
                        subject.time == map.num_time.time
                ) {
                    Some(subject) => subject,
                    None => {
                        subject_created = true;
                        let subject = TchrSubject {
                            raw: map.cell.text.clone(),
                            num: map.num_time.num,
                            time: map.num_time.time.clone(),
                            name: name.to_string(),
                            format: Format::Remote,
                            groups: groups.clone(),
                            cabinet: None,
                        };
                        day.subjects.push(subject);
                        day.subjects.last_mut().unwrap()
                    }
                };

                if !subject_created {
                    for this_group in groups {
                        let existing = subject.groups.iter().find(
                            |group| group.to_string() == this_group.to_string()
                        );
                        if existing.is_none() {
                            subject.groups.push(this_group);
                        }
                    }
                }
            }
        }

        let page = TchrPage {
            raw:       base_weekday.cell.text.to_owned(),
            raw_types: vec![raw::Type::TchrRWeekly],
            sc_type:   Type::Weekly,
            date: {
                let wkd_range = self.weekday_date_range.as_ref().unwrap();

                let start = wkd_range.first().unwrap().date;
                let end = wkd_range.last().unwrap().date;

                start..=end
            },
            num_time_mappings: None,
            teachers,
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }
}