use log::info;

use crate::data::schedule::{
    remote::table::SubjectMapping,
    Subject,
    Day,
    Group,
    Page, Format
};
use super::super::{teacher, subject};


/// # 3rd, final step of parsing remote schedule
#[derive(Debug, Clone)]
pub struct Parser {
    schema: Vec<Vec<SubjectMapping>>,

    page: Option<Page>
}
impl Parser {
    pub fn new(schema: Vec<Vec<SubjectMapping>>, page: Option<Page>) -> Parser {
        Parser { schema, page, }
    }

    pub fn from_schema(schema: Vec<Vec<SubjectMapping>>) -> Parser {
        Parser::new(schema, None)
    }

    pub fn page(&mut self) -> Option<&Page> {
        if self.page.is_some() {
            return Some(self.page.as_ref().unwrap())
        }

        for group_row in self.schema.iter() {
            for subject in group_row.iter() {

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
                    let teachers = teacher::extract_from_end(subject_string);

                    let subject = Subject {
                        raw: subject.cell.text.clone(),
                        num: subject.num_time.num,
                        time: subject.num_time.time.clone(),
                        name: subject_string.clone(),
                        format: Format::Remote,
                        teachers,
                        cabinet: None,
                    };

                    info!("{:#?}", subject);
                }
            }
            todo!();
        }

        todo!()
    }
}