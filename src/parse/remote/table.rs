use derive_new::new;
use chrono::NaiveDate;
use std::{collections::HashMap, f64::consts::E, ops::{ControlFlow, Range, RangeInclusive}, sync::Arc};

use crate::{data::{
    schedule::raw::{
        remote::{self, table::{
            NumTime, SubjectMapping, TchrSubjectMapping, WeekdayDate
        }},
        table::{self, Teacher}
    }, 
    Weekday
}, parse::{group, teacher}, REGEX};
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

            let mut string_weekday = date::remove(&cell.text).to_lowercase();
            let mut weekday_chars: Vec<char> = string_weekday.chars().collect();
            weekday_chars[0] = weekday_chars[0].to_uppercase().nth(0).unwrap();
            string_weekday = weekday_chars.into_iter().collect();

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
            mapping::Parser::from_schema(grouped_mappings, None)
        );

        Some(self.mapping.as_mut().unwrap())
    }

    pub fn mapping_v2(&mut self) -> Option<&mut mapping::Parser> {
        let wkd_row = self.weekday_date_row()?.clone();

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
        let mut y_hits: Vec<table::RangeHit> = vec![];
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
                if cell.rowspan > 0 {
                    let y_hit = table::RangeHit {
                        by: cell.clone(),
                        x_rng: cell.x..cell.colspan+cell.x,
                        y_rng: cell.y..cell.rowspan+cell.y,
                        is_done: false
                    };
                    y_hits.push(y_hit);
                }
            }
        }

        for (index, row) in schema.into_iter().enumerate() {
            let is_teacher_row = row.get(0).map(|cell| cell.x > 2).unwrap_or(false);
            // y == index
            let y = row.get(0).unwrap().y;

            let mut is_in_group_rng = current_group_row_range.as_ref().map_or_else(
                || false,
                |rng| rng.contains(&y)
            );

            let mut is_in_numtime_rng = current_numtime_row_range.as_ref().map_or_else(
                || false,
                |rng| rng.contains(&y)
            );

            if !is_in_group_rng {
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

                // first cell could also be a group identifier
                // ("1-кДД-43" for example)
                let first_cell = row.iter().find(|cell| cell.x == 0);
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
                let second_cell = row.iter().find(|cell| cell.x == 1);
                if second_cell.is_none() { continue; }

                // third cell could also be a subject time
                // ("8:30-9:55" for example)
                let third_cell = row.iter().find(|cell| cell.x == 2);
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
                    hits_for_this_pos = y_hits.iter().filter(
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

                    if cell.text.is_empty() || is_online_identifier {
                        continue;
                    }

                    let existing_maps;

                    if !is_teacher_row {
                        existing_maps = current_numtime_mappings.iter_mut().filter(
                            |map| 
                                map.group.valid == current_group.as_ref().unwrap().valid &&
                                map.weekday_date.weekday == current_wkd.unwrap().weekday &&
                                map.weekday_date.date == current_wkd.unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.text == cell.text &&
                                map.cell.x == cell.x
                        ).collect::<Vec<&mut SubjectMapping>>();
                    } else {
                        existing_maps = current_numtime_mappings.iter_mut().filter(
                            |map| 
                                map.group.valid == current_group.as_ref().unwrap().valid &&
                                map.weekday_date.weekday == current_wkd.unwrap().weekday &&
                                map.weekday_date.date == current_wkd.unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.x == cell.x
                        ).collect::<Vec<&mut SubjectMapping>>();
                    }

                    if existing_maps.is_empty() {
                        let map = SubjectMapping {
                            cell: cell.clone(),
                            group: current_group.as_ref().unwrap().clone(),
                            num_time: current_numtime.as_ref().unwrap().clone(),
                            weekday_date: current_wkd.unwrap().clone(),
                        };

                        current_numtime_mappings.push(map);
                    } else {
                        for existing_map in existing_maps {
                            if !is_teacher_row || existing_map.cell.x != cell.x || cell.x != x {
                                continue;
                            }

                            //existing_map.cell.x != cell.x

                            let has_another_teacher = REGEX.end_teacher.find(
                                &existing_map.cell.text
                            ).is_some();
    
                            let teacher_string = teacher::shorten(&cell.text);
    
                            if !existing_map.cell.text.ends_with(".") {
                                existing_map.cell.text += ".";
                            }
    
                            if has_another_teacher {
                                existing_map.cell.text += ",";
                            }
    
                            if teacher_string.is_some() {
                                existing_map.cell.text += " ";
                                existing_map.cell.text += teacher_string.as_ref().unwrap();
                            }
                        }
                    }
                } else if !hits_for_this_pos.is_empty() {
                    if is_teacher_row {
                        continue;
                    }

                    for hit in hits_for_this_pos {
                        let existing_map = current_numtime_mappings.iter_mut().find(
                            |map| 
                                map.group.valid == current_group.as_ref().unwrap().valid &&
                                map.weekday_date.weekday == current_wkd.unwrap().weekday &&
                                map.weekday_date.date == current_wkd.unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.text == hit.by.text
                        );

                        if existing_map.is_some() {
                            continue;
                        }

                        let map = SubjectMapping {
                            cell: hit.by.clone(),
                            group: current_group.as_ref().unwrap().clone(),
                            num_time: current_numtime.as_ref().unwrap().clone(),
                            weekday_date: current_wkd.unwrap().clone(),
                        };
    
                        current_numtime_mappings.push(map);
                    }
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

        for group_map in all_mappings.iter_mut() {
            group_map.sort_by(|mapa, mapb|
                mapa.weekday_date.weekday.partial_cmp(
                    &mapb.weekday_date.weekday
                ).unwrap()
            )
        }
        
        self.mapping = Some(
            mapping::Parser::from_schema(all_mappings, Some(wkd_row))
        );

        Some(self.mapping.as_mut().unwrap())
    }

    pub fn take_mapping(&mut self) -> Option<mapping::Parser> {
        self.mapping.take()
    }
}

/// # 2nd step of parsing remote schedule
#[derive(new, Debug, Clone)]
pub struct TchrParser {
    table: table::Body,

    base_date: Option<NaiveDate>,
    teachers_row: Option<Vec<remote::table::Teacher>>,
    mapping: Option<mapping::TchrParser>,
}
impl TchrParser {
    pub fn from_table(table: table::Body) -> Self {
        Self::new(table, None, None, None)
    }

    pub fn base_date(&mut self) -> Option<&NaiveDate> {
        if self.base_date.is_some() {
            return Some(self.base_date.as_ref().unwrap())
        }

        for row in self.table.schema.iter() {
            if let Some(probably_date_header) = row.get(0) {
                if let Some(date) = date::parse_dmy(&probably_date_header.text) {
                    self.base_date = Some(date);
                    return Some(self.base_date.as_ref().unwrap())
                }
            }
        }

        None
    }

    pub fn teachers_row(&mut self) -> Option<&Vec<remote::table::Teacher>> {
        if self.teachers_row.is_some() {
            return Some(self.teachers_row.as_ref().unwrap())
        }

        let mut row = vec![];

        for (index, cell) in self.table.schema.get(0)?.iter().enumerate() {
            if cell.text.is_empty() { continue; }

            let mut string_teacher = teacher::shorten(&cell.text);
            if string_teacher.is_none() { continue; }

            let tchr = remote::table::Teacher::new(
                cell.clone(),
                index,
                Teacher {
                    raw: cell.text.clone(),
                    valid: string_teacher.unwrap()
                }
            );
            row.push(tchr);
        }

        self.teachers_row = Some(row);


        Some(self.teachers_row.as_ref().unwrap())
    }

    fn row_for_y(y: usize, schema: &Vec<Vec<table::Cell>>) -> Option<&Vec<table::Cell>> {
        let mut i: i32 = -1;
        loop {
            i += 1;
            //
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

    pub fn mapping(&mut self) -> Option<&mut mapping::TchrParser> {
        self.teachers_row()?;

        let mut current_weekday: Option<WeekdayDate> = None;
        let mut current_weekday_row_range: Option<Range<usize>> = None;
        let mut seen_weekdays: Vec<WeekdayDate> = vec![];

        let mut current_numtime: Option<NumTime> = None;
        let mut current_numtime_row_range: Option<Range<usize>> = None;

        let mut current_teacher: Option<table::Teacher> = None;
        let mut current_teacher_col_range: Option<Range<usize>> = None;

        let mut current_weekday_mappings: Vec<TchrSubjectMapping> = vec![];
        let mut current_numtime_mappings: Vec<TchrSubjectMapping> = vec![];


        let mut all_mappings: Vec<Vec<TchrSubjectMapping>> = vec![];
        let mut y_hits: Vec<table::RangeHit> = vec![];

        let teacher_row = Arc::new(self.teachers_row.take()?);

        let schema = {
            let mut v = vec![];
            v.append(&mut self.table.schema);
            v
        };
        let schema_len = schema.len();


        for row in schema.iter() {
            for cell in row {
                if cell.rowspan > 0 {
                    let y_hit = table::RangeHit {
                        by: cell.clone(),
                        x_rng: cell.x..cell.colspan+cell.x,
                        y_rng: cell.y..cell.rowspan+cell.y,
                        is_done: false
                    };
                    y_hits.push(y_hit);
                }
            }
        }

        for (index, row) in schema.into_iter().enumerate() {
            let is_last = index == schema_len - 1;
            // y == index
            let y = row.get(0).unwrap().y;
            let is_subject_name_row = current_numtime_row_range.as_ref().map(
                |rng| y == rng.end-1
            ).unwrap_or(false);

            let is_in_weekday_rng = current_weekday_row_range.as_ref().map_or_else(
                || false,
                |rng| rng.contains(&y)
            );

            if !is_in_weekday_rng {
                if !current_numtime_mappings.is_empty() {
                    current_weekday_mappings.append(&mut current_numtime_mappings);
                }

                if !current_weekday_mappings.is_empty() {
                    all_mappings.push({
                        let mut v = vec![];
                        v.append(&mut current_weekday_mappings);
                        v
                    });
                }

                // first cell could be a weekday identifier
                // ("Понедельник 27.05.2024" for example)
                let Some(first_cell) = row.iter().find(|cell| cell.x == 0) else {
                    continue;
                };

                // we are probably on a row
                // with a different weekday
                let raw_weekday_date = &first_cell.text;
                let mut string_weekday = date::remove(&raw_weekday_date).to_lowercase();
                let mut weekday_chars: Vec<char> = string_weekday.chars().collect();
                if weekday_chars.len() < 1 { continue; }
                weekday_chars[0] = weekday_chars[0].to_uppercase().nth(0).unwrap();
                string_weekday = weekday_chars.into_iter().collect();
    
                let weekday = Weekday::guess(&string_weekday);
                if weekday.is_none() { continue; }
    
                let date = date::parse_dmy(&raw_weekday_date);
                if date.is_none() { continue; }
    
                let wkd_date = WeekdayDate::new(
                    first_cell.clone(),
                    index,
                    weekday.unwrap(),
                    date.unwrap()
                );
                current_weekday = Some(wkd_date.clone());
                current_weekday_row_range = Some(
                    first_cell.y..first_cell.y+first_cell.rowspan
                );
                seen_weekdays.push(wkd_date);
            }

            let is_in_numtime_rng = current_numtime_row_range.as_ref().map_or_else(
                || false,
                |rng| rng.contains(&y)
            );

            if !is_in_numtime_rng {
                if !current_numtime_mappings.is_empty() {
                    current_weekday_mappings.append(&mut current_numtime_mappings);
                }

                // second cell could also be a subject number
                // ("1" for example)
                let second_cell = row.iter().find(|cell| cell.x == 1);
                if second_cell.is_none() { continue; }

                // third cell could also be a subject time
                // ("8:30-9:55" for example)
                let third_cell = row.iter().find(|cell| cell.x == 2);
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
                current_numtime_row_range = Some(y..third_cell.unwrap().rowspan + y);
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
                    hits_for_this_pos = y_hits.iter().filter(
                        |hit| hit.x_rng.contains(&x) && hit.y_rng.contains(&y)
                    ).collect::<Vec<&table::RangeHit>>();
                }

                if cell.is_none() && hits_for_this_pos.is_empty() {
                    break;
                }

                let current_teacher = teacher_row
                    .iter()
                    .find(|tchr| (
                        // x..colspan+x
                        tchr.cell.x..tchr.cell.colspan+tchr.cell.x
                    ).contains(&x));

                if current_teacher.is_none() {
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

                    if cell.text.is_empty() || is_online_identifier {
                        continue;
                    }

                    let existing_maps;

                    if !is_subject_name_row {
                        existing_maps = current_numtime_mappings.iter_mut().filter(
                            |map| 
                                map.teacher.valid == current_teacher.as_ref().unwrap().teacher.valid &&
                                map.weekday_date.weekday == current_weekday.as_ref().unwrap().weekday &&
                                map.weekday_date.date == current_weekday.as_ref().unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.text == cell.text &&
                                map.cell.x == cell.x
                        ).collect::<Vec<&mut TchrSubjectMapping>>();
                    } else {
                        existing_maps = current_numtime_mappings.iter_mut().filter(
                            |map| 
                                map.teacher.valid == current_teacher.as_ref().unwrap().teacher.valid &&
                                map.weekday_date.weekday == current_weekday.as_ref().unwrap().weekday &&
                                map.weekday_date.date == current_weekday.as_ref().unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.x == cell.x
                        ).collect::<Vec<&mut TchrSubjectMapping>>();
                    }

                    if existing_maps.is_empty() {
                        let map = TchrSubjectMapping {
                            cell: cell.clone(),
                            teacher: current_teacher.as_ref().unwrap().teacher.clone(),
                            num_time: current_numtime.as_ref().unwrap().clone(),
                            weekday_date: current_weekday.as_ref().unwrap().clone(),
                        };

                        current_numtime_mappings.push(map);
                    } else {
                        for existing_map in existing_maps {
                            if !is_subject_name_row || existing_map.cell.x != cell.x || cell.x != x {
                                continue;
                            }
                            // a genius way to separate group names from a subject name
                            existing_map.cell.text += "*&^%$#@!FUCKING_SEPARATOR!@#$%^&*";
                            existing_map.cell.text += &cell.text;
                        }
                    }
                } else if !hits_for_this_pos.is_empty() {
                    if is_subject_name_row {
                        continue;
                    }

                    for hit in hits_for_this_pos {
                        let existing_map = current_numtime_mappings.iter_mut().find(
                            |map| 
                                map.teacher.valid == current_teacher.as_ref().unwrap().teacher.valid &&
                                map.weekday_date.weekday == current_weekday.as_ref().unwrap().weekday &&
                                map.weekday_date.date == current_weekday.as_ref().unwrap().date &&
                                map.num_time.num == current_numtime.as_ref().unwrap().num &&
                                map.num_time.time == current_numtime.as_ref().unwrap().time &&
                                map.cell.text == hit.by.text
                        );

                        if existing_map.is_some() {
                            continue;
                        }

                        let map = TchrSubjectMapping {
                            cell: hit.by.clone(),
                            teacher: current_teacher.as_ref().unwrap().teacher.clone(),
                            num_time: current_numtime.as_ref().unwrap().clone(),
                            weekday_date: current_weekday.as_ref().unwrap().clone(),
                        };
    
                        current_numtime_mappings.push(map);
                    }
                }
            }
        }

        if !current_numtime_mappings.is_empty() {
            current_weekday_mappings.append(&mut current_numtime_mappings);
        }

        if !current_weekday_mappings.is_empty() {
            all_mappings.push({
                let mut v = vec![];
                v.append(&mut current_weekday_mappings);
                v
            });
        }

        for tchr_map in all_mappings.iter_mut() {
            tchr_map.sort_by(|mapa, mapb|
                mapa.weekday_date.weekday.partial_cmp(
                    &mapb.weekday_date.weekday
                ).unwrap()
            )
        }

        seen_weekdays.sort_by(|wkda, wkdb|
            wkda.weekday.partial_cmp(
                &wkdb.weekday
            ).unwrap()
        );

        self.mapping = Some(
            mapping::TchrParser::from_schema(all_mappings, Some(seen_weekdays))
        );

        Some(self.mapping.as_mut().unwrap())
    }

    pub fn take_mapping(&mut self) -> Option<mapping::TchrParser> {
        self.mapping.take()
    }
}