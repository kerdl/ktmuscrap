use std::ops::RangeBounds;

use log::info;
use chrono::NaiveDate;
use itertools::Itertools;

use crate::{data::{
    schedule::{
        raw::table, 
        remote::table::{NumTime, WeekdayDate, SubjectMapping},
        Page,
        Group,
        Day,
        Subject,
    },
    weekday::Weekday,
}, parse::group};
use super::super::{date, time, num};


/// # 2nd step of parsing remote schedule
#[derive(Debug, Clone)]
pub struct Parser {
    table: table::Body,

    base_date: Option<NaiveDate>,
    weekday_date_row: Option<Vec<WeekdayDate>>,
    num_time_row: Option<Vec<NumTime>>,
    mappings: Option<Vec<Vec<SubjectMapping>>>,
}
impl Parser {
    pub fn new(
        table: table::Body, 
        base_date: Option<NaiveDate>,
        weekday_date_row: Option<Vec<WeekdayDate>>,
        num_time_row: Option<Vec<NumTime>>,
        mappings: Option<Vec<Vec<SubjectMapping>>>,
    ) -> Parser {

        Parser {
            table,
            base_date,
            weekday_date_row,
            num_time_row,
            mappings
        }
    }

    pub fn from_table(table: table::Body) -> Parser {
        Parser::new(table, None, None, None, None)
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

    pub fn mappings(&mut self) -> Option<&Vec<Vec<SubjectMapping>>> {

        self.weekday_date_row()?;
        self.num_time_row()?;

        let mut grouped_mappings: Vec<Vec<SubjectMapping>> = vec![];
        let mut hits: Vec<table::Hit> = vec![];

        for row in self.table.schema.iter() {

            let y = row.get(0)?.y;

            let group = group::parse(&row.get(0)?.text);
            if group.is_none() { continue; }
            let group = group.unwrap();

            let mut mappings: Vec<SubjectMapping> = vec![];


            let mut past_cells_count = 0;

            let mut weekday_index = 0;
            let mut num_time_index = 0;

            for (index, cell) in row[1..].iter().enumerate() {
                let x = cell.x;

                for hit in hits.iter_mut() {

                    if hit.is_done { continue; }
                    if hit.at_y != y { continue; }

                    let last_map_cell_rng = { mappings.last().map(|map| map.cell.width()..x).unwrap_or(0..x) };

                    if !last_map_cell_rng.contains(&hit.at_x) {
                        continue;
                    }

                    let mut y_neighbour = None;

                    for mappings in grouped_mappings.iter().rev() {

                        let map = mappings.iter().find(|&map| 
                            last_map_cell_rng.contains(&map.cell.x)
                            && map.cell.y == hit.by.y
                        );
                        if map.is_none() { continue; }

                        y_neighbour = map;
                        break;
                    }

                    if y_neighbour.is_none() { continue; }
                    let y_neighbour = y_neighbour.unwrap();

                    let mapping = SubjectMapping::new(
                        y_neighbour.cell.clone(),
                        group.clone(),
                        y_neighbour.num_time.clone(), 
                        y_neighbour.weekday_date.clone(), 
                        y_neighbour.subject.clone()
                    );

                    mappings.push(mapping);

                    hit.done();
                }

                let weekday_date = {
                    self.weekday_date_row.as_ref().unwrap()
                    .get(weekday_index)?
                };
                let num_time = {
                    self.num_time_row.as_ref().unwrap()
                    .get(num_time_index)?
                };
                let subject = &cell.text;


                let mapping = SubjectMapping::new(
                    cell.clone(),
                    group.clone(),
                    num_time.clone(), 
                    weekday_date.clone(), 
                    subject.clone()
                );

                mappings.push(mapping);

                if cell.hits_next_rows() {
                    let mut future_y = y + 1;

                    for _ in 0..cell.hits() {
                        let hit = table::Hit {
                            by:      cell, 
                            at_x:    x, 
                            at_y:    future_y,
                            is_done: false,
                        };

                        hits.push(hit);

                        future_y += 1;
                    }
                }


                past_cells_count += cell.width();
                weekday_index = {
                    past_cells_count / weekday_date.cell.colspan as usize
                };
                num_time_index = {
                    past_cells_count
                };
            }

            grouped_mappings.push(mappings);
        }

        self.mappings = Some(grouped_mappings);

        Some(self.mappings.as_ref().unwrap())
    }
}