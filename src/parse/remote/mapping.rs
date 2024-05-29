use derive_new::new;

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

        for numtime_row in self.schema.iter() {
            let teacher_name = &numtime_row.get(0)?.teacher;

            let mut days: Vec<TchrDay> = vec![];
            let mut subjects: Vec<TchrSubject> = vec![];

            for (index, subject) in numtime_row.iter().enumerate() {
                let next_subject = numtime_row.get(index + 1);

                let (groups, subject_name) = subject.cell.text.split_once(
                    "*&^%$#@!FUCKING_SEPARATOR!@#$%^&*"
                ).unwrap();

                if groups.is_empty() || subject_name.is_empty() { continue; }

                let groups = group::parse_multiple(groups);

                let mut parsed_subject = TchrSubject {
                    raw:      subject.cell.text.clone(),
                    num:      subject.num_time.num,
                    time:     subject.num_time.time.clone(),
                    name:     subject_name.to_string(),
                    format:   Format::Remote,
                    groups,
                    cabinet:  None,
                };

                let existing_subjects = subjects.iter_mut().filter(
                    |subj| subj.name == parsed_subject.name &&
                    subj.num == parsed_subject.num &&
                    subj.time == parsed_subject.time
                ).collect::<Vec<&mut TchrSubject>>();

                if !existing_subjects.is_empty() {
                    for existing_subject in existing_subjects {
                        existing_subject.groups.append(&mut parsed_subject.groups);
                    }
                } else {
                    subjects.push(parsed_subject);
                }

                let is_changing_weekday = {
                    next_subject.is_some()
                    && next_subject.as_ref().unwrap().weekday_date != subject.weekday_date
                };

                let was_last = {
                    next_subject.is_none()
                };

                if (is_changing_weekday || was_last) && !subjects.is_empty() {
                    let day = TchrDay {
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

            let teacher = TchrTeacher {
                raw:  teacher_name.raw.to_owned(),
                name: teacher_name.valid.to_owned(),
                days
            };

            teachers.push(teacher);
        }

        let page = TchrPage {
            raw:       base_weekday.cell.text.to_owned(),
            raw_types: vec![raw::Type::RWeekly],
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