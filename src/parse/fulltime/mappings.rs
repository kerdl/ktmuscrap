use derive_new::new;
use log::warn;

use crate::{
    data::schedule::{
        raw::{
            self,
            fulltime::table::{
                SubjectMapping, 
                GroupSubjects, 
            }
        },
        Type,
        Page, 
        Group,
        Day,
        Subject, 
        Format
    },
    parse::cabinet
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