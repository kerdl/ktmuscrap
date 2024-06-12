use derive_new::new;
use chrono::{NaiveTime, NaiveDate};
use itertools::Itertools;
use std::{collections::HashMap, ops::Range, sync::{Arc, RwLock}};

use crate::data::{
    schedule::raw::{
        self,
        NumTime,
        fulltime::{
            html::{HeaderTable, HeaderSpanTable},
            table::{
                NumTimeWithOrigin,
                SubjectMapping,
                GroupSubjects,
                TeacherSubjects,
                CellType,
                WeekdayWithOrigin,
            }
        },
        table,
    },
    Weekday,
};
use super::{
    mappings::Parser as MappingsParser, 
    mappings::TchrDailyParser as TchrDailyMappingsParser, 
    mappings::TchrWeeklyParser as TchrWeeklyMappingsParser, 
    super::{
        date,
        time,
        group,
        teacher
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
                                    time.clone()
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

#[derive(new, Debug, Clone)]
pub struct TchrDailyParser {
    header_table: HeaderSpanTable,

    mappings: Option<TchrDailyMappingsParser>
}
impl TchrDailyParser {
    pub fn from_header_tables(header_tables: HeaderSpanTable) -> Self {
        Self::new(header_tables, None)
    }

    pub fn mappings(
        &mut self,
        num_time_mappings: Option<HashMap<Weekday, HashMap<u32, Range<NaiveTime>>>>
    ) -> Option<&mut TchrDailyMappingsParser> {
        if self.mappings.is_some() {
            return Some(self.mappings.as_mut().unwrap())
        }

        let mut teacher_maps: Vec<TeacherSubjects> = vec![];
        let Some(date_range) = date::parse_dmy_range(&self.header_table.header) else {
            return None;
        };
        let mut weekday: Option<WeekdayWithOrigin> = None;
        let mut num_row: Vec<NumTimeWithOrigin> = vec![];

        for (row_i, row) in self.header_table.table.iter().enumerate() {
            match row_i {
                0 => {
                    for cell in row {
                        let Some(wkd) = Weekday::guess(&cell.text) else {
                            continue;
                        };
                        weekday = Some(WeekdayWithOrigin::new(cell.text.clone(), wkd));
                    }
                    continue;
                },
                1 => {
                    for cell in row {
                        let Ok(parsed) = cell.text.parse::<u32>() else {
                            continue;
                        };
                        let time = weekday.as_ref().map(
                            |wkd| num_time_mappings.as_ref().map(|maps| maps.get(&wkd.guessed).map(
                                |map| map.get(&parsed).cloned()
                            ))
                        ).flatten().flatten().flatten();
                        let num_time = NumTime::new(parsed, time.clone());
                        let with_origin = NumTimeWithOrigin::new(
                            num_time,
                            cell.clone()
                        );
                        num_row.push(with_origin);
                    }
                    continue;
                },
                _ => {},
            }

            if row.len() < 2 { continue; }

            let teacher = row[0].text.clone();
            let mut subjects = vec![];
            for cell in row[1..].iter() {
                let Some(num_time_with_origin) = num_row.iter().find(
                    |map| (map.cell.x..map.cell.colspan+map.cell.x).contains(&cell.x)
                ) else {
                    continue;
                };

                subjects.push(SubjectMapping::new(
                    cell.text.clone(),
                    weekday.as_ref().unwrap().clone(),
                    num_time_with_origin.num_time.clone()
                ));
            }

            teacher_maps.push(TeacherSubjects::new(
                table::Teacher::new(teacher.clone(), teacher.clone()),
                date_range.clone(),
                subjects
            ));
        }
        
        let parser = TchrDailyMappingsParser::from_teachers_subjects(
            teacher_maps,
            self.header_table.header.clone(),
        );

        self.mappings = Some(parser);

        Some(self.mappings.as_mut().unwrap())
    }

    pub fn take_mappings(&mut self) -> Option<TchrDailyMappingsParser> {
        self.mappings.take()
    }
}

#[derive(new, Debug, Clone)]
pub struct TchrWeeklyParser {
    header_tables: Vec<HeaderTable>,

    mappings: Option<TchrWeeklyMappingsParser>
}
impl TchrWeeklyParser {
    pub fn from_header_tables(header_tables: Vec<HeaderTable>) -> Self {
        Self::new(header_tables, None)
    }

    pub fn mappings(
        &mut self,
        num_time_mappings: Option<HashMap<Weekday, HashMap<u32, Range<NaiveTime>>>>
    ) -> Option<&mut TchrWeeklyMappingsParser> {
        if self.mappings.is_some() {
            return Some(self.mappings.as_mut().unwrap())
        }

        let mut tasks = vec![];

        let mut teachers_maps: Vec<TeacherSubjects> = vec![];

        let header_tables = {
            let mut v = vec![];
            v.append(&mut self.header_tables);
            v
        };

        for teacher_section in header_tables.into_iter() {
            if teacher_section.table.len() < 2 { continue };

            let raw_teacher = &teacher_section.header;
            let Some(valid_teacher) = teacher::parse(&teacher_section.header) else { continue };
            let Some(date_range) = date::parse_dmy_range(&teacher_section.header) else { continue };

            let teacher = raw::table::Teacher::new(
                raw_teacher.clone(), 
                valid_teacher
            );

            let num_time_maps_clone = num_time_mappings.as_ref().cloned();
            let task = std::thread::spawn(move || -> TeacherSubjects {
                let table_header = &teacher_section.table[0];

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

                let table_body = &teacher_section.table[1..];

                'row: for row in table_body.iter() {
                    let mut num: Option<u32> = None;
                    let mut time: Option<NaiveTime> = None;
    
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
                                time = time::parse_hm(cell);

                                if time.is_none() {
                                    continue 'row;
                                }
                            }
                            CellType::Subject => {
                                let name = cell.clone();
                                let weekday = weekdays_map.get(&index).unwrap().clone();

                                let mut numtime_map_override = None;
                                if let Some(numtime_maps) = &num_time_maps_clone {
                                    if let Some(wkd_map) = numtime_maps.get(&weekday.guessed) {
                                        numtime_map_override = wkd_map.get(&num.unwrap()).cloned()
                                    }
                                }
                                let usable_time = if let Some(map_override) = numtime_map_override {
                                    Some(map_override)
                                } else {
                                    time.as_ref().map(|time| time.clone()..time.clone())
                                };

                                let num_time = NumTime::new(
                                    num.unwrap(), 
                                    usable_time
                                );
    
                                let map = SubjectMapping::new(name, weekday, num_time);
                                subject_maps.push(map);
                            }
                        }
                    }
                }
    
                let mut teacher_map = TeacherSubjects::new(
                    teacher, 
                    date_range, 
                    subject_maps
                );
    
                teacher_map.subjects.sort_by(
                    |map_a, map_b| map_a.weekday.guessed.cmp(&map_b.weekday.guessed)
                );

                teacher_map
            });

            tasks.push(task);
        }

        for result in tasks {
            let teacher_map = result.join().unwrap();
            teachers_maps.push(teacher_map);
        }
        
        let parser = TchrWeeklyMappingsParser::from_teacher_subjects(
            teachers_maps
        );

        self.mappings = Some(parser);

        Some(self.mappings.as_mut().unwrap())
    }

    pub fn take_mappings(&mut self) -> Option<TchrWeeklyMappingsParser> {
        self.mappings.take()
    }
}