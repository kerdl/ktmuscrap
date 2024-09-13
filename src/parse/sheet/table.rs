use chrono::{Datelike, NaiveDate};
use crate::data::{schedule::raw::table::{self, XCord, YCord, Width, XRange}, Weekday};
use crate::{parse, regexes};


#[derive(thiserror::Error, Debug)]
#[error("table schema parsing error")]
pub enum ParsingError {
    NoDatesRow
}


pub struct Parser {
    pub schema: Vec<Vec<table::Cell>>
}
impl Parser {
    pub fn from_schema(schema: Vec<Vec<table::Cell>>) -> Self {
        Self { schema }
    }

    pub fn date_row(&self) -> Option<&Vec<table::Cell>> {
        for row in self.schema.iter() {
            for cell in row.iter() {
                let weekday_match = regexes().whole_short_weekday.find(&cell.text);
                if weekday_match.is_some() {
                    let date_match = regexes().date.find(&cell.text);
                    if date_match.is_some() {
                        return Some(row);
                    }
                }
            }
        }

        None
    }

    pub fn date_ranges(&self) -> Option<Vec<table::Date>> {
        let row = self.date_row()?;
        let mut opt_ranges: Vec<table::OptDate> = vec![];
        let mut ranges: Vec<table::Date> = vec![];

        for (idx, cell) in row.iter().enumerate() {
            let weekday_matches = regexes()
                .whole_short_weekday
                .find_iter(&cell.text)
                .collect::<Vec<regex::Match>>();
            let date_matches = regexes()
                .date
                .find_iter(&cell.text)
                .collect::<Vec<regex::Match>>();

            if !date_matches.is_empty() {
                let start = parse::date::whole(
                    date_matches.get(0).unwrap().as_str()
                ).unwrap();
                let end = parse::date::whole(
                    date_matches.last().unwrap().as_str()
                ).unwrap();

                let opt_date = table::OptDate {
                    raw: &cell.text,
                    parsed: Some(start..end),
                    range: (cell.x())..(cell.x() + cell.width() - 1)
                };

                opt_ranges.push(opt_date);
            } else if !weekday_matches.is_empty() {
                let opt_date = table::OptDate {
                    raw: &cell.text,
                    parsed: None,
                    range: (cell.x())..(cell.x() + cell.width() - 1)
                };

                opt_ranges.push(opt_date);
            }
        }

        'row: for row in self.schema.iter() {
            for cell in row.iter() {
                if cell.text.contains("3 ПОТОК") {
                    println!();
                    break 'row;
                }
            }
        }

        // can't modify in-place
        // because fuck rust
        let mut to_be_filled = vec![];

        for (idx, opt_range) in opt_ranges.iter().enumerate() {
            let is_last = idx == opt_ranges.len() - 1;
            if opt_range.parsed.is_some() { continue; }

            let is_reversed;
            let nearest_some = if !is_last {
                is_reversed = false;
                let iter = opt_ranges.iter().enumerate();
                let mut value = None;
                for (nearest_idx, nearest_opt_range) in iter {
                    if nearest_opt_range.parsed.is_some() {
                        value = Some((nearest_idx, nearest_opt_range));
                        break;
                    }
                }
                value
            } else {
                is_reversed = true;
                let iter = opt_ranges.iter().rev().enumerate();
                let mut value = None;
                for (nearest_idx, nearest_opt_range) in iter {
                    if nearest_opt_range.parsed.is_some() {
                        value = Some((nearest_idx, nearest_opt_range));
                        break;
                    }
                }
                value
            };

            let Some(nearest_some) = nearest_some else { continue };
            let days_shift = nearest_some.0;
            let date = nearest_some.1;

            let filled_date = if !is_reversed {
                date.parsed.as_ref().unwrap().start
                    .checked_sub_days(chrono::Days::new(days_shift as u64))
            } else {
                date.parsed.as_ref().unwrap().end
                    .checked_add_days(chrono::Days::new(days_shift as u64))
            };
            
            to_be_filled.push((idx, filled_date));
        }

        for fill in to_be_filled {
            let idx = fill.0;
            let date = fill.1;
            opt_ranges.get_mut(idx).unwrap().parsed = date.map(|date| date..date);
        }

        for opt_range in opt_ranges.into_iter() {
            let Some(date) = opt_range.to_date() else {
                continue
            };
            ranges.push(date);
        }

        Some(ranges)
    }

    pub async fn parse(&self) -> Result<(), ParsingError> {
        let Some(dates) = self.date_ranges() else {
            return Err(ParsingError::NoDatesRow)
        };

        let mut current_formation: Option<table::Formation> = None;

        for row in self.schema.iter() {
            break;
            let y = row.get(0).unwrap().y;

            let mut x = -1;
            loop {
                x += 1;
            }
        }

        Ok(())
    }

    pub fn row_for_y<Cell: YCord>(y: usize, schema: &Vec<Vec<Cell>>) -> Option<&Vec<Cell>> {
        let mut i: i32 = -1;
        loop {
            i += 1;

            let i = i as usize;
            let Some(row) = schema.get(i) else { break; };
            let next_row = schema.get(i + 1);

            let Some(row_y) = row.get(0).map(|cell| cell.y()) else {
                break;
            };
            let next_row_y = next_row.map(
                |row| row.get(0).map(|cell| cell.y())
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

    pub fn cell_for_x<Cell: XCord + XRange>(x: usize, row: &Vec<Cell>) -> Option<&Cell> {
        for cell in row {
            if cell.x() == x || cell.x_range().contains(&x) {
                return Some(cell)
            }
        }

        None
    }
}