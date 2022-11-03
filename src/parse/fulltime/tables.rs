use derive_new::new;
use chrono::NaiveTime;
use std::{collections::HashMap, ops::Range};

use crate::data::{
    schedule::{
        raw,
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
    weekday::Weekday
};
use super::{
    mappings::Parser as MappingsParser, 
    super::{
        date,
        time,
        group,
    }
};


#[derive(new)]
pub struct Parser<'a> {
    sc_type: &'a raw::Type,
    header_tables: Vec<HeaderTable>,

    mappings: Option<MappingsParser<'a>>
}
impl<'a> Parser<'a> {
    pub fn from_header_tables(header_tables: Vec<HeaderTable>, sc_type: &raw::Type) -> Parser {
        Parser::new(sc_type, header_tables, None)
    }

    pub fn mappings(&'a mut self) -> Option<&mut MappingsParser> {
        if self.mappings.is_some() {
            return Some(self.mappings.as_mut().unwrap())
        }

        let mut groups_maps: Vec<GroupSubjects> = vec![];

        for group_section in self.header_tables.iter() {

            let raw_group = &group_section.header;
            let valid_group = group::parse(&group_section.header);
            let date_range = date::parse_dmy_range(&group_section.header);

            if valid_group.is_none() {
                continue;
            }

            if date_range.is_none() {
                continue;
            }

            let group = raw::table::Group::new(
                raw_group.clone(), 
                valid_group.unwrap()
            );


            let mut weekdays_map: HashMap<usize, WeekdayWithOrigin> = HashMap::new();
            let mut subject_maps: Vec<SubjectMapping> = vec![];

            for (index, cell) in {
                group_section
                .table.get(0).unwrap()
                .iter().enumerate()
            } {
                if let Some(weekday) = Weekday::guess(cell) {
                    weekdays_map.insert(index, WeekdayWithOrigin::new(cell.to_owned(), weekday));
                }
            }

            for row in group_section.table[1..].iter() {

                let mut num: Option<u32> = None;
                let mut time: Option<Range<NaiveTime>> = None;

                for (index, cell) in row.iter().enumerate() {
                    let cell_type = CellType::from_index(index);

                    match cell_type {
                        CellType::Num => {
                            num = Some(cell.parse().unwrap())
                        }
                        CellType::Time => {
                            time = time::parse_range_hm(cell)
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
                date_range.unwrap(), 
                subject_maps
            );

            group_map.subjects.sort();

            groups_maps.push(group_map);
        }

        let parser = MappingsParser::from_groups_subjects(
            groups_maps,
            &self.sc_type
        );

        self.mappings = Some(parser);

        Some(self.mappings.as_mut().unwrap())
    }
}