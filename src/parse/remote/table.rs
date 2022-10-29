use chrono::NaiveDate;
use log::info;

use crate::data::schedule::raw::table;
use super::super::date;


/// # 2nd, final step of parsing remote schedule
#[derive(Debug, Clone)]
pub struct Parser {
    schema: table::Body,

    base_date: Option<NaiveDate>
}
impl Parser {
    pub fn new(table: table::Body, base_date: Option<NaiveDate>) -> Parser {
        Parser { schema: table, base_date }
    }

    pub fn from_table(table: table::Body) -> Parser {
        Parser::new(table, None)
    }

    pub fn base_date(&mut self) -> Option<&NaiveDate> {
        if self.base_date.is_some() {
            return Some(self.base_date.as_ref().unwrap())
        }

        let weekday_row = self.schema.schema.get(0)?;

        for weekday in weekday_row.iter() {
            let dmy = date::parse_dmy(&weekday.text);

            if dmy.is_some() {
                self.base_date = dmy;
                return Some(self.base_date.as_ref().unwrap())
            }
        }

        None
    }

    pub fn dick(&self) {
        //info!("{:?}", self.table);
    }
}