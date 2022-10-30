use chrono::NaiveDate;
use log::info;

use crate::data::{
    schedule::{
        raw::table, 
        remote::table::{NumTime, WeekdayDate},
        Group,
        Day,
        Subject,
    },
    weekday::Weekday,
};
use super::super::{date, time, num};


/// # 2nd, final step of parsing remote schedule
#[derive(Debug, Clone)]
pub struct Parser {
    table: table::Body,

    base_date: Option<NaiveDate>,
    weekday_date_row: Option<Vec<WeekdayDate>>,
    num_time_row: Option<Vec<NumTime>>,
}
impl Parser {
    pub fn new(
        table: table::Body, 
        base_date: Option<NaiveDate>,
        weekday_date_row: Option<Vec<WeekdayDate>>,
        num_time_row: Option<Vec<NumTime>>,
    ) -> Parser {

        Parser {
            table,
            base_date,
            weekday_date_row,
            num_time_row
        }
    }

    pub fn from_table(table: table::Body) -> Parser {
        Parser::new(table, None, None, None)
    }

    pub fn base_date(&mut self) -> Option<&NaiveDate> {
        if self.base_date.is_some() {
            return Some(self.base_date.as_ref().unwrap())
        }

        let weekday_row = self.table.schema.get(0)?;

        for weekday in weekday_row.iter() {
            let dmy = date::parse_dmy(&weekday.text);

            if dmy.is_some() {
                self.base_date = dmy;
                return Some(self.base_date.as_ref().unwrap())
            }
        }

        None
    }

    pub fn weekday_date_row(&mut self) -> Option<&Vec<WeekdayDate>> {
        if self.weekday_date_row.is_some() {
            return Some(self.weekday_date_row.as_ref().unwrap())
        }

        let mut row = vec![];

        for (index, cell) in self.table.schema.get(0)?.iter().enumerate() {

            if cell.text.is_empty() { continue; }

            let string_weekday = date::remove(&cell.text);
            let weekday = Weekday::guess(&string_weekday);
            if weekday.is_none() { continue; }

            let date = date::parse_dmy(&cell.text);
            if date.is_none() { continue; }

            let wkd_date = WeekdayDate::new(
                cell.clone(),
                index,
                weekday.unwrap(),
                date.unwrap()
            );
            row.push(wkd_date);
        }

        self.weekday_date_row = Some(row);

        Some(self.weekday_date_row.as_ref().unwrap())
    }

    pub fn num_time_row(&mut self) -> Option<&Vec<NumTime>> {
        if self.num_time_row.is_some() {
            return Some(self.num_time_row.as_ref().unwrap())
        }

        let mut row = vec![];

        for (index, cell) in self.table.schema.get(1)?.iter().enumerate() {

            if cell.text.is_empty() { continue; }

            let raw_num = time::remove(&cell.text);
            let num = num::parse(&raw_num);
            if num.is_none() { continue; }

            let time = time::parse_range_hm(&cell.text);
            if time.is_none() { continue; }

            let num_time = NumTime::new(
                cell.clone(), 
                index, 
                num.unwrap(), 
                time.unwrap()
            );
            row.push(num_time);
        }

        self.num_time_row = Some(row);

        Some(self.num_time_row.as_ref().unwrap())
    }

    pub fn group(&mut self) {

    }

    pub fn dick(&self) {
        info!("{:?}", self.table);
    }
}