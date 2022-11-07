use derive_new::new;

use crate::data::schedule::{
    raw,
    remote::table::{
        SubjectMapping,
        WeekdayDate
    },
    Type,
    Subject,
    Day,
    Group,
    Page, 
    Format
};
use super::super::{
    teacher,
    subject
};


/// # 3rd, final step of parsing remote schedule
#[derive(new, Debug, Clone)]
pub struct Parser {
    schema: Vec<Vec<SubjectMapping>>,

    pub page: Option<Page>
}
impl Parser {
    pub fn from_schema(schema: Vec<Vec<SubjectMapping>>) -> Parser {
        Parser::new(schema, None)
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

                    let subject = Subject {
                        raw:      subject.cell.text.clone(),
                        num:      subject.num_time.num,
                        time:     subject.num_time.time.clone(),
                        name:     subject_string.clone(),
                        format:   Format::Remote,
                        teachers,
                        cabinet:  None,
                    };

                    if !subject.is_fulltime_window() {
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

                start..end
            },
            groups,
        };

        self.page = Some(page);

        Some(self.page.as_ref().unwrap())
    }
}