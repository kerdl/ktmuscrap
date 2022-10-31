use crate::data::schedule::{
    remote::table::SubjectMapping,
    Subject,
    Day,
    Group,
    Page
};


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
                /* 
                Subject {
                    raw,
                    num,
                    time,
                    name,
                    format,
                    teachers,
                    cabinet
                }
                */
            }
        }

        todo!()
    }
}