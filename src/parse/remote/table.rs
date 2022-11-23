use log::debug;
use derive_new::new;
use chrono::NaiveDate;
use std::{sync::{Arc, RwLock}, time::Instant};

use crate::{data::{
    schedule::{
        raw::{
            table, remote::table::{
                NumTime, 
                WeekdayDate, 
                SubjectMapping, 
            },
        }, 
    },
    Weekday,
}, parse::group};
use super::{
    super::{date, time, num},
    mapping
};


/// # 2nd step of parsing remote schedule
#[derive(new, Debug, Clone)]
pub struct Parser {
    table: table::Body,

    base_date: Option<NaiveDate>,
    weekday_date_row: Option<Vec<WeekdayDate>>,
    num_time_row: Option<Vec<NumTime>>,
    mapping: Option<mapping::Parser>,
}
impl Parser {
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

    pub fn mapping(&mut self) -> Option<&mut mapping::Parser> {

        self.weekday_date_row()?;
        self.num_time_row()?;

        let mut grouped_mappings: Vec<Vec<SubjectMapping>> = vec![];
        let mut hits: Vec<table::Hit> = vec![];


        let weekday_row = Arc::new(self.weekday_date_row.take()?);
        let num_time_row = Arc::new(self.num_time_row.take()?);

        let schema = {
            let mut v = vec![];
            v.append(&mut self.table.schema);
            v
        };

        for (index, row) in schema.into_iter().enumerate() {
            let y = row.get(0).unwrap().y;

            let raw_group = &row.get(0).unwrap().text;
            let valid_group = group::parse(&raw_group);

            if valid_group.is_none() { continue; }

            let group = table::Group::new(
                raw_group.to_string(),
                valid_group.unwrap().to_string()
            );

            let mut mappings: Vec<SubjectMapping> = vec![];


            for cell in row[1..].iter() {
                let x = cell.x;

                for hit in hits.iter_mut() {

                    if hit.is_done { continue; }
                    if hit.at_y != y { continue; }

                    let last_map_cell_rng = {
                        mappings.last().map(|map| 
                            (map.cell.x + map.cell.width())..x
                        ).unwrap_or(0..x)
                    };

                    if !last_map_cell_rng.contains(&hit.at_x) {
                        continue;
                    }

                    let mut y_neighbour = None;

                    for maps in grouped_mappings.iter().rev() {

                        let map = maps.iter().find(|&map| 
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
                    );

                    mappings.push(mapping);

                    hit.done();
                }
                
                let last_mapping_x = mappings.last().map(
                    |map| map.cell.x
                ).unwrap_or(0);

                let last_mapping_colspan = mappings.last().map(
                    |map| map.cell.colspan
                ).unwrap_or(0);

                let past_cells_count = {
                    last_mapping_x + last_mapping_colspan.checked_sub(1).unwrap_or(0)
                };

                let weekday_date = {
                    weekday_row.iter().find(|wkd| 
                        (
                            (wkd.cell.x)
                            ..
                            (wkd.cell.x + wkd.cell.colspan)
                        ).contains(&(past_cells_count + 1))
                    ).unwrap()
                };
                let num_time = {
                    num_time_row.get(past_cells_count).unwrap()
                };


                let mapping = SubjectMapping::new(
                    cell.clone(),
                    group.clone(),
                    num_time.clone(), 
                    weekday_date.clone(),
                );

                mappings.push(mapping);

                if cell.hits_next_rows() {
                    let mut future_y = y + 1;

                    for _ in 0..cell.hits() {
                        let hit = table::Hit {
                            by:      cell.clone(), 
                            at_x:    x, 
                            at_y:    future_y,
                            is_done: false,
                        };

                        hits.push(hit);

                        future_y += 1;
                    }
                }
            }

            let maps = {
                let mut v = vec![];
                v.append(&mut mappings);
                v
            };

            grouped_mappings.push(maps);
        }

        self.mapping = Some(
            mapping::Parser::from_schema(grouped_mappings)
        );

        Some(self.mapping.as_mut().unwrap())
    }

    pub fn take_mapping(&mut self) -> Option<mapping::Parser> {
        self.mapping.take()
    }
}
