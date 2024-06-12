use derive_new::new;
use itertools::Itertools;
use log::warn;

use chrono::NaiveTime;
use std::{ops::Range, collections::HashMap};

use crate::{
    data::{schedule::{
        raw::{
            self,
            fulltime::table::{
                GroupSubjects, SubjectMapping, TeacherSubjects
            }
        },
        Day,
        Format,
        Group,
        Page,
        Subgroup,
        Subject,
        TchrDay,
        TchrPage,
        TchrSubject,
        TchrTeacher,
        Type
    }, Weekday},
    parse::{cabinet, group}
};
use super::super::teacher;


#[derive(new, Debug, Clone)]
pub struct Parser {
    sc_type: raw::Type,
    pub groups_subjects: Vec<GroupSubjects>,

    pub page: Option<Page>
}
impl Parser {
    pub fn from_groups_subjects(
        groups_subjects: Vec<GroupSubjects>,
        sc_type: raw::Type
    ) -> Parser {
        Parser::new(sc_type, groups_subjects, None)
    }

    pub fn page(&mut self) -> Option<&Page> {
        let mut groups: Vec<Group> = vec![];

        for group_map in self.groups_subjects.iter() {
            let mut days: Vec<Day> = vec![];
            let mut subjects: Vec<Subject> = vec![];

            let filtered = {
                group_map.subjects.iter()
                .filter(|subject| !subject.is_empty())
                .collect::<Vec<&SubjectMapping>>()
            };

            for (index, subject) in filtered.iter().enumerate() {
                let next_subject = filtered.get(index + 1);

                let mut name = subject.name.clone();

                let cabinet = cabinet::extract_from_end(&mut name);
                let teachers = teacher::extract_from_end(&mut name);

                let parsed_subject = Subject {
                    raw:    subject.name.clone(),
                    num:    subject.num_time.num,
                    time:   subject.num_time.time.clone(),
                    name,
                    format: Format::Fulltime,
                    teachers,
                    cabinet
                };

                subjects.push(parsed_subject);

                let is_changing_weekday = {
                    next_subject.is_some()
                    && next_subject.unwrap().weekday.guessed != subject.weekday.guessed
                };
                let was_last = {
                    next_subject.is_none()
                };

                if (is_changing_weekday || was_last) && !subjects.is_empty() {

                    let raw = subject.weekday.raw.clone();
                    let weekday = subject.weekday.guessed.clone();
                    let date = {
                        subject.weekday.guessed
                        .date_from_range(&group_map.date_range)
                        .unwrap()
                    };

                    let day = Day {
                        raw,
                        weekday,
                        date,
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
                raw: group_map.group.raw.clone(),
                name: group_map.group.valid.clone(),
                days
            };

            groups.push(group);
        }

        if groups.is_empty() {
            warn!("no groups were found on {} schedule", self.sc_type);
            return None;
        }

        let page = Page {
            raw:       groups.get(0).unwrap().raw.clone(),
            raw_types: vec![self.sc_type.clone()],
            sc_type: {
                match self.sc_type {
                    raw::Type::FtDaily => Type::Daily,
                    raw::Type::FtWeekly => Type::Weekly,
                    _ => unreachable!()
                }
            },
            date: {
                let mut starts = vec![];
                let mut ends = vec![];

                for group in groups.iter() {
                    starts.push(&group.days.first().unwrap().date);
                    ends.push(&group.days.last().unwrap().date);
                }

                let start = **starts.iter().min().unwrap();
                let end = **ends.iter().max().unwrap();

                match self.sc_type {
                    raw::Type::FtWeekly => start..=end,
                    raw::Type::FtDaily => start..=start,
                    _ => unreachable!()
                }
            },
            groups
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }
}

#[derive(new, Debug, Clone)]
pub struct TchrDailyParser {
    pub teachers_subjects: Vec<TeacherSubjects>,
    pub header: String,

    pub page: Option<TchrPage>
}
impl TchrDailyParser {
    pub fn from_teachers_subjects(
        teachers_subjects: Vec<TeacherSubjects>,
        header: String,
    ) -> Self {
        Self::new(teachers_subjects, header, None)
    }

    pub fn page(&mut self) -> Option<&TchrPage> {
        let mut teachers: Vec<TchrTeacher> = vec![];

        for tchr_map in self.teachers_subjects.iter() {
            let mut days: Vec<TchrDay> = vec![];
            let mut subjects: Vec<TchrSubject> = vec![];

            let filtered = {
                tchr_map.subjects.iter()
                .filter(|subject| !subject.is_empty())
                .collect::<Vec<&SubjectMapping>>()
            };

            for (index, subject) in filtered.iter().enumerate() {
                let next_subject = filtered.get(index + 1);

                let name_clone = subject.name.clone();
                let mut groups = vec![];
                let raw_groups;
                let mut cabinet = None;

                let all_raw_groups_and_cabient = name_clone.split_once(" ");
                if all_raw_groups_and_cabient.is_none() {
                    raw_groups = name_clone.split(",").collect_vec();
                } else {
                    let (all_raw_groups, cab) = all_raw_groups_and_cabient.unwrap();
                    cabinet = Some(cab.to_string());
                    raw_groups = all_raw_groups.split(",").collect_vec();
                }

                for grp in raw_groups {
                    let Some(parsed_group) = group::parse(grp) else {
                        continue;
                    };
                    let subgroup = Subgroup {
                        group: parsed_group,
                        subgroup: None,
                    };
                    groups.push(subgroup);
                }
                
                let parsed_subject = TchrSubject {
                    raw:    subject.name.clone(),
                    num:    subject.num_time.num,
                    time:   subject.num_time.time.clone(),
                    name:   "".to_string(),
                    format: Format::Fulltime,
                    groups,
                    cabinet: cabinet
                };

                subjects.push(parsed_subject);

                let is_changing_weekday = {
                    next_subject.is_some()
                    && next_subject.unwrap().weekday.guessed != subject.weekday.guessed
                };
                let was_last = {
                    next_subject.is_none()
                };

                if (is_changing_weekday || was_last) && !subjects.is_empty() {
                    let raw = subject.weekday.raw.clone();
                    let weekday = subject.weekday.guessed.clone();
                    let date = {
                        subject.weekday.guessed
                        .date_from_range(&tchr_map.date_range)
                        .unwrap()
                    };

                    let day = TchrDay {
                        raw,
                        weekday,
                        date,
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
                raw: tchr_map.teacher.raw.clone(),
                name: tchr_map.teacher.valid.clone(),
                days
            };

            teachers.push(teacher);
        }

        if teachers.is_empty() {
            warn!("no teachers were found on {} schedule", raw::Type::TchrFtDaily);
            return None;
        }

        let page = TchrPage {
            raw: self.header.clone(),
            raw_types: vec![raw::Type::TchrFtDaily],
            sc_type: Type::Daily,
            date: {
                let mut starts = vec![];

                for teacher in teachers.iter() {
                    starts.push(&teacher.days.first().unwrap().date);
                }

                let start = **starts.iter().min().unwrap();

                start..=start
            },
            num_time_mappings: None,
            teachers
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }
}

#[derive(new, Debug, Clone)]
pub struct TchrWeeklyParser {
    pub teacher_subjects: Vec<TeacherSubjects>,

    pub page: Option<TchrPage>
}
impl TchrWeeklyParser {
    pub fn from_teacher_subjects(
        teacher_subjects: Vec<TeacherSubjects>
    ) -> Self {
        Self::new(teacher_subjects, None)
    }

    pub fn page(&mut self) -> Option<&TchrPage> {
        let mut teachers: Vec<TchrTeacher> = vec![];

        for teacher_map in self.teacher_subjects.iter() {
            let mut days: Vec<TchrDay> = vec![];
            let mut subjects: Vec<TchrSubject> = vec![];

            let filtered = {
                teacher_map.subjects.iter()
                .filter(|subject| !subject.is_empty())
                .collect::<Vec<&SubjectMapping>>()
            };

            for (index, subject) in filtered.iter().enumerate() {
                let next_subject = filtered.get(index + 1);

                let mut name = subject.name.clone();

                let cabinet = cabinet::extract_from_end(&mut name);
                let groups = group::extract_from_start(&mut name).iter().map(
                    |grp| Subgroup { group: group::parse(&grp).unwrap(), subgroup: None }
                ).collect_vec();

                let parsed_subject = TchrSubject {
                    raw:    subject.name.clone(),
                    num:    subject.num_time.num,
                    time:   subject.num_time.time.clone(),
                    name,
                    format: Format::Fulltime,
                    groups,
                    cabinet
                };

                subjects.push(parsed_subject);

                let is_changing_weekday = {
                    next_subject.is_some()
                    && next_subject.unwrap().weekday.guessed != subject.weekday.guessed
                };
                let was_last = {
                    next_subject.is_none()
                };

                if (is_changing_weekday || was_last) && !subjects.is_empty() {
                    let raw = subject.weekday.raw.clone();
                    let weekday = subject.weekday.guessed.clone();
                    let date = {
                        subject.weekday.guessed
                        .date_from_range(&teacher_map.date_range)
                        .unwrap()
                    };

                    let day = TchrDay {
                        raw,
                        weekday,
                        date,
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
                raw: teacher_map.teacher.raw.clone(),
                name: teacher_map.teacher.valid.clone(),
                days
            };

            teachers.push(teacher);
        }

        if teachers.is_empty() {
            warn!("no teachers were found on {} schedule", raw::Type::TchrFtWeekly);
            return None;
        }

        let page = TchrPage {
            raw: teachers.get(0).unwrap().raw.clone(),
            raw_types: vec![raw::Type::TchrFtWeekly],
            sc_type: Type::Weekly,
            date: {
                let mut starts = vec![];
                let mut ends = vec![];

                for teacher in teachers.iter() {
                    starts.push(&teacher.days.first().unwrap().date);
                    ends.push(&teacher.days.last().unwrap().date);
                }

                let start = **starts.iter().min().unwrap();
                let end = **ends.iter().max().unwrap();

                start..=end
            },
            num_time_mappings: None,
            teachers
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }
}