use derive_new::new;
use chrono::NaiveDate;
use std::{collections::HashMap, f64::consts::E, ops::{ControlFlow, Range}, sync::Arc};

use crate::{data::{
    schedule::raw::{
        remote::table::{
            NumTime, SubjectMapping, WeekdayDate
        }, table
    }, 
    Weekday
}, parse::group, REGEX};
use super::{
    super::{date, time, num},
    mapping
};

enum AddWay {
    Push,
    Splice(Range<usize>),
}

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
            let num = num::parse_from_digit_containig(&raw_num);
            if num.is_none() { continue; }

            let time = time::parse_range_hm(&cell.text);
            if time.is_none() { continue; }

            let num_time = NumTime::new(
                num.unwrap(), 
                time.unwrap()
            );
            row.push(num_time);
        }

        self.num_time_row = Some(row);

        Some(self.num_time_row.as_ref().unwrap())
    }

    fn row_for_y(y: usize, schema: &Vec<Vec<table::Cell>>) -> Option<&Vec<table::Cell>> {
        let mut i: i32 = -1;
        loop {
            i += 1;

            let i = i as usize;
            let Some(row) = schema.get(i) else { break; };
            let next_row = schema.get(i + 1);

            let Some(row_y) = row.get(0).map(|cell| cell.y) else {
                break;
            };
            let next_row_y = next_row.map(
                |row| row.get(0).map(|cell| cell.y)
            ).flatten();

            if next_row_y.is_none() && y <= row_y {
                return Some(row)
            }
            
            let Some(next_row_y) = next_row_y else { break; };

            if (row_y..next_row_y).contains(&y) {
                return Some(row);
            }
        }

        None
    }

    fn cell_for_x(x: usize, row: &Vec<table::Cell>) -> Option<&table::Cell> {
        for cell in row {
            if cell.x == x || (cell.x..(cell.colspan+cell.x)).contains(&x) {
                return Some(cell)
            }
        }

        None
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

            let mut cell_index = 0;
            let mut last_x = 0;

            loop {
                let cell = row[1..].get(cell_index);
                let x = cell.map(|cell| cell.x).unwrap_or(last_x + 1);

                let is_out_of_range = cell.is_none();
                let mut has_uncompleted_hits_for_this_y = false;

                loop {
                    let mut hits_done = 0;

                    for hit in hits.iter_mut() {
                        if hit.is_done { continue; }
                        if hit.at_y != y { continue; }
    
                        let last_map_cell_rng = {
                            mappings.last().map(|map| 
                                (map.cell.x + map.cell.width())..x
                            ).unwrap_or(0..x)
                        };
    
                        if !last_map_cell_rng.contains(&hit.at_x) {
                            has_uncompleted_hits_for_this_y = true;
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
                        hits_done += 1;
                    }

                    if hits_done < 1 {
                        break;
                    }
                }

                let flow;

                if !is_out_of_range {
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
                        cell.cloned().unwrap(),
                        group.clone(),
                        num_time.clone(), 
                        weekday_date.clone(),
                    );
    
                    mappings.push(mapping);
    
                    if cell.as_ref().unwrap().hits_next_rows() {
                        let mut future_y = y + 1;
    
                        for _ in 0..cell.as_ref().unwrap().hits() {
                            let hit = table::Hit {
                                by:      cell.cloned().unwrap(),
                                at_x:    x, 
                                at_y:    future_y,
                                is_done: false,
                            };

                            let mut add_way = AddWay::Push;
    
                            if !hits.is_empty() {
                                for (index, current) in hits.iter().enumerate() {
                                    let next = hits.get(index + 1);
                                    if next.is_none() {
                                        break;
                                    }
                                    let next = next.unwrap();

                                    if next.at_y == hit.at_y && next.at_x > hit.at_x {
                                        add_way = AddWay::Splice(index + 1..index + 1);
                                        break;
                                    } else {
                                        continue;
                                    }
                                }
                            }
    
                            match add_way {
                                AddWay::Push => { hits.push(hit); },
                                AddWay::Splice(range) => { hits.splice(range, [hit]); }
                            }

                            future_y += 1;
                        }
                    }

                    flow = ControlFlow::Continue(())
                } else if has_uncompleted_hits_for_this_y {
                    flow = ControlFlow::Continue(())
                } else {
                    flow = ControlFlow::Break(())
                }

                cell_index += 1;
                last_x = x;

                match flow {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => break,
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

    pub fn mapping_v2(&mut self) -> Option<&mut mapping::Parser> {
        self.weekday_date_row()?;

        /////////////////////////////////////
        // all rows that we currently process
        // are related to this group
        let mut current_group: Option<table::Group> = None;
        // range of rows that are related
        // to the current group
        let mut current_group_row_range: Option<Range<usize>> = None;
        let mut current_group_mappings: Vec<SubjectMapping> = vec![];
        /////////////////////////////////////
        

        /////////////////////////////////////
        let mut current_numtime_mappings: Vec<SubjectMapping> = vec![];
        /////////////////////////////////////


        /////////////////////////////////////
        // all rows that we currently process
        // are related to this number and time
        let mut current_numtime: Option<NumTime> = None;
        // range of rows that are related
        // to the current subject number and time
        let mut current_numtime_row_range: Option<Range<usize>> = None;
        /////////////////////////////////////


        let mut all_mappings: Vec<Vec<SubjectMapping>> = vec![];
        let mut hits: Vec<table::RangeHit> = vec![];
        //let mut y_hits: HashMap<usize, Vec<table::RangeHit>> = HashMap::new();

        let weekday_row = Arc::new(self.weekday_date_row.take()?);

        let schema = {
            let mut v = vec![];
            v.append(&mut self.table.schema);
            v
        };
        let schema_len = schema.len();


        for row in schema.iter() {
            for cell in row {
                if cell.rowspan > 0 || cell.colspan > 0 {
                    let hit = table::RangeHit {
                        by: cell.clone(),
                        x_rng: cell.x..cell.colspan+cell.x,
                        y_rng: cell.y..cell.rowspan+cell.y,
                        is_done: false
                    };
                    hits.push(hit);
                }
            }
        }

        for (index, row) in schema.into_iter().enumerate() {
            let is_last = index == schema_len - 1;
            let is_teacher_row = row.get(0).map(|cell| cell.x > 2).unwrap_or(false);
            // y == index
            let y = row.get(0).unwrap().y;

            let mut first_cell = None;
            let mut second_cell = None;
            let mut third_cell = None;

            let mut is_in_group_rng = current_group_row_range.as_ref().map_or_else(
                || false,
                |rng| rng.contains(&y)
            );

            let mut is_in_numtime_rng = current_numtime_row_range.as_ref().map_or_else(
                || false,
                |rng| rng.contains(&y)
            );


            if !is_in_group_rng {
                if !current_group_mappings.is_empty() {
                    all_mappings.push({
                        let mut v = vec![];
                        v.append(&mut current_group_mappings);
                        v
                    });
                }

                // first cell could also be a group identifier
                // ("1-кДД-43" for example)
                first_cell = row.iter().find(|cell| cell.x == 0);
                if first_cell.is_none() { continue; }

                // we are probably on a row
                // with different group
                let raw_group = &first_cell.unwrap().text;
                let valid_group = group::parse(raw_group);
                if valid_group.is_none() { continue; }

                current_group = Some(table::Group::new(
                    raw_group.to_string(),
                    valid_group.unwrap().to_string()
                ));
                current_group_row_range = Some(y..first_cell.unwrap().rowspan + y);
                is_in_group_rng = true;
            }

            if !is_in_numtime_rng {
                if !current_numtime_mappings.is_empty() {
                    current_group_mappings.append(&mut current_numtime_mappings);
                }

                // second cell could also be a subject number
                // ("1" for example)
                second_cell = row.iter().find(|cell| cell.x == 1);
                if second_cell.is_none() { continue; }

                // third cell could also be a subject time
                // ("8:30-9:55" for example)
                third_cell = row.iter().find(|cell| cell.x == 2);
                if third_cell.is_none() { continue; }

                // we are probably on a row
                // with different subject number and time
                let raw_num = &second_cell.unwrap().text;
                let raw_time = &third_cell.unwrap().text;
                let valid_num = num::parse_from_digits_only(raw_num);
                let valid_time = time::parse_range_hm(raw_time);
                if valid_num.is_none() || valid_time.is_none() { continue; }
                current_numtime = Some(NumTime::new(
                    valid_num.unwrap() as u32,
                    valid_time.unwrap()
                ));
                current_numtime_row_range = Some(y..second_cell.unwrap().rowspan + y);
                is_in_numtime_rng = true;
            }

            let mut x: i32 = -1;

            loop {
                x += 1;

                // x == 0: is a group name
                // x == 1: is a subject number
                // x == 2: is a subject time range
                if x < 3 {
                    continue;
                }

                let mut cell = None;
                let x = x as usize;

                let mut hits_for_this_pos = vec![];

                if let Some(c) = Parser::cell_for_x(x, &row) {
                    cell = Some(c);
                } else {
                    hits_for_this_pos = hits.iter().filter(
                        |hit| hit.x_rng.contains(&x) && hit.y_rng.contains(&y)
                    ).collect::<Vec<&table::RangeHit>>();
                }

                if cell.is_none() && hits_for_this_pos.is_empty() {
                    break;
                }

                let current_wkd = weekday_row
                    .iter()
                    .find(|wkd| (
                        // x..colspan+x
                        wkd.cell.x..wkd.cell.colspan+wkd.cell.x
                    ).contains(&x));

                if current_wkd.is_none() {
                    continue;
                }

                if let Some(cell) = cell {
                    let is_online_identifier = {
                        let has_digits = REGEX.digit.find(&cell.text).is_some();
                        // "Онлайн "
                        let online_word = REGEX.digit.replace_all(&cell.text, "");
                        let is_similar = crate::data::schedule::ONLINE_IDENTIFIER_CORPUS
                            .search(&online_word, 0.7)
                            .first()
                            .is_some();
                        has_digits && is_similar
                    };

                    println!(
                        "x={},\ny={},\ngroup {:?},\nnumtime {:?},\nwkd {:?},\nis_online_identifier {:?},\n{:?}\n",
                        x,
                        y,
                        current_group,
                        current_numtime,
                        current_wkd,
                        is_online_identifier,
                        cell
                    );

                    if cell.text.is_empty() || is_online_identifier {
                        continue;
                    }

                    let existing_map;

                    if !is_teacher_row {
                        existing_map = current_numtime_mappings.iter_mut().find(
                            |map| 
                                map.group.valid == current_group.as_ref().unwrap().valid &&
                                map.weekday_date.weekday == current_wkd.unwrap().weekday &&
                                map.weekday_date.date == current_wkd.unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.text == cell.text
                        );
                    } else {
                        existing_map = current_numtime_mappings.iter_mut().find(
                            |map| 
                                map.group.valid == current_group.as_ref().unwrap().valid &&
                                map.weekday_date.weekday == current_wkd.unwrap().weekday &&
                                map.weekday_date.date == current_wkd.unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time
                        );
                    }


                    if existing_map.is_none() {
                        let map = SubjectMapping {
                            cell: cell.clone(),
                            group: current_group.as_ref().unwrap().clone(),
                            num_time: current_numtime.as_ref().unwrap().clone(),
                            weekday_date: current_wkd.unwrap().clone(),
                        };
    
                        current_numtime_mappings.push(map);
                    } else {
                        let existing_map = existing_map.unwrap();

                        if !is_teacher_row || cell.x != x {
                            continue;
                        }

                        let mut teacher_string = "".to_string();
                        let teacher_parts = cell.text.split(" ").collect::<Vec<&str>>();

                        if teacher_parts.len() == 1 {
                            teacher_string+= &teacher_parts[0];
                        } else if teacher_parts.len() == 2 {
                            teacher_string += &teacher_parts[0];
                            teacher_string += " ";
                            teacher_string += &teacher_parts[1].chars().nth(0).unwrap().to_string();
                            teacher_string += ".";
                        } else if teacher_parts.len() > 2 {
                            teacher_string += &teacher_parts[0];
                            teacher_string += " ";
                            teacher_string += &teacher_parts[1].chars().nth(0).unwrap().to_string();
                            teacher_string += ".";
                            teacher_string += &teacher_parts[2].chars().nth(0).unwrap().to_string();
                            teacher_string += ".";
                        }

                        if !existing_map.cell.text.ends_with(".") {
                            existing_map.cell.text += ".";
                        }

                        existing_map.cell.text += " ";
                        existing_map.cell.text += &teacher_string;
                    }
                } else if !hits_for_this_pos.is_empty() {
                    println!(
                        "x={},\ny={},\ngroup {:?},\nnumtime {:?},\nwkd {:?},\n{:?}\n",
                        x,
                        y,
                        current_group,
                        current_numtime,
                        current_wkd,
                        hits_for_this_pos
                    );
                }
            }
        }

        if !current_numtime_mappings.is_empty() {
            current_group_mappings.append(&mut current_numtime_mappings);
        }

        if !current_group_mappings.is_empty() {
            all_mappings.push({
                let mut v = vec![];
                v.append(&mut current_group_mappings);
                v
            });
        }

        self.mapping = Some(
            mapping::Parser::from_schema(all_mappings)
        );

        Some(self.mapping.as_mut().unwrap())
    }

    pub fn take_mapping(&mut self) -> Option<mapping::Parser> {
        self.mapping.take()
    }
}
