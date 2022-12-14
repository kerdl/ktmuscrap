use derive_new::new;
use chrono::NaiveTime;
use std::{collections::HashMap, ops::Range, sync::Arc};

use crate::data::{
    schedule::raw::{
        self,
        fulltime::{
            html::HeaderTable, 
            table::{
                NumTime,
                SubjectMapping, 
                GroupSubjects, 
                CellType,
                WeekdayWithOrigin
            }
        }
    },
    Weekday
};
use super::{
    mappings::Parser as MappingsParser, 
    super::{
        date,
        time,
        group,
    }
};


#[derive(new, Debug, Clone)]
pub struct Parser {
    sc_type: raw::Type,
    header_tables: Vec<HeaderTable>,

    mappings: Option<MappingsParser>
}
impl Parser {
    pub fn from_header_tables(header_tables: Vec<HeaderTable>, sc_type: raw::Type) -> Parser {
        Parser::new(sc_type, header_tables, None)
    }

    pub fn mappings(&mut self) -> Option<&mut MappingsParser> {
        if self.mappings.is_some() {
            return Some(self.mappings.as_mut().unwrap())
        }

        let mut tasks = vec![];

        let mut groups_maps: Vec<GroupSubjects> = vec![];

        let header_tables = {
            let mut v = vec![];
            v.append(&mut self.header_tables);
            v
        };

        for group_section in header_tables.into_iter() {
            if group_section.table.len() < 2 { continue };

            let raw_group = &group_section.header;
            let Some(valid_group) = group::parse(&group_section.header) else { continue };
            let Some(date_range) = date::parse_dmy_range(&group_section.header) else { continue };

            let group = raw::table::Group::new(
                raw_group.clone(), 
                valid_group
            );

            let task = std::thread::spawn(move || -> GroupSubjects {
                let table_header = &group_section.table[0];

                let mut weekdays_map: HashMap<usize, WeekdayWithOrigin> = HashMap::new();
                let mut subject_maps: Vec<SubjectMapping> = vec![];

                for (index, cell) in table_header.iter().enumerate() {
                    if let Some(weekday) = Weekday::guess(cell) {

                        let w_origin = WeekdayWithOrigin::new(
                            cell.to_owned(), 
                            weekday
                        );

                        weekdays_map.insert(
                            index, 
                            w_origin
                        );
                    }
                }

                let table_body = &group_section.table[1..];

                'row: for row in table_body.iter() {
                    let mut num: Option<u32> = None;
                    let mut time: Option<Range<NaiveTime>> = None;
    
                    for (index, cell) in row.iter().enumerate() {
                        let cell_type = CellType::from_index(index);
    
                        match cell_type {
                            CellType::Num => {
                                num = cell.parse().ok();

                                if num.is_none() {
                                    continue 'row;
                                }
                            }
                            CellType::Time => {
                                time = time::parse_range_hm(cell);

                                if time.is_none() {
                                    continue 'row;
                                }
                            }
                            CellType::Subject => {
                                let name = cell.clone();
                                let weekday = weekdays_map.get(&index).unwrap().clone();
                                let num_time = NumTime::new(
                                    num.unwrap(), 
                                    time.as_ref().unwrap().clone()
                                );
    
                                let map = SubjectMapping::new(name, weekday, num_time);
                                subject_maps.push(map);
                            }
                        }
                    }
                }
    
                let mut group_map = GroupSubjects::new(
                    group, 
                    date_range, 
                    subject_maps
                );
    
                group_map.subjects.sort_by(
                    |map_a, map_b| map_a.weekday.guessed.cmp(&map_b.weekday.guessed)
                );

                group_map
            });

            tasks.push(task);
        }

        for result in tasks {
            let group_map = result.join().unwrap();
            groups_maps.push(group_map);
        }
        
        let parser = MappingsParser::from_groups_subjects(
            groups_maps,
            self.sc_type.clone()
        );

        self.mappings = Some(parser);

        Some(self.mappings.as_mut().unwrap())
    }

    pub fn take_mappings(&mut self) -> Option<MappingsParser> {
        self.mappings.take()
    }
}