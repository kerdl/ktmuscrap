use chrono::{Datelike, NaiveDate};
use crate::data::{schedule::{
    self,
    raw::{
        self,
        table::{
            self,
            Width,
            XCord,
            XRange,
            YCord,
            YRange
        }
    }
}, Weekday};
use crate::{parse, regexes};


#[derive(thiserror::Error, Debug)]
#[error("table schema parsing error")]
pub enum ParsingError {
    NoDatesRow
}


pub struct Parser {
    pub schema: Vec<Vec<table::Cell>>,
    pub kind: raw::Kind
}
impl Parser {
    pub fn from_schema(schema: Vec<Vec<table::Cell>>, kind: raw::Kind) -> Self {
        Self { schema, kind }
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

        let mut formations: Vec<schedule::Formation> = vec![];

        // either a group or a teacher
        let mut current_formation: Option<table::Formation> = None;
        // subject number
        let mut num_counter = 0;
        // list of cells that expand onto the next rows
        let mut y_hits: Vec<table::RangeHit> = vec![];

        for row in self.schema.iter() {
            let y = row.get(0).unwrap().y;
            num_counter += 1;

            let mut is_in_formation_range = current_formation
                .as_ref()
                .map_or(false, |form| form.y_range().contains(&y));

            if !is_in_formation_range {
                // first cell is a formation identifier
                // ("1-кДД-43" for example)
                let Some(first_cell) = row
                    .iter()
                    .find(|cell| cell.x == 0) else { continue };

                let Some(valid_formation) = (match self.kind {
                    raw::Kind::Groups => parse::group::validate(&first_cell.text),
                    raw::Kind::Teachers => parse::teacher::validate(&first_cell.text),
                }) else { continue };

                current_formation = Some(table::Formation {
                    kind: self.kind.as_attender(),
                    raw: &first_cell.text,
                    valid: valid_formation,
                    range: first_cell.y_range()
                });
                is_in_formation_range = true;
                // switch to a new formation resets
                // the subject number counter
                num_counter = 1;
            }

            let mut x = -1;
            loop {
                x += 1;

                let mut cell = None;
                let x = x as usize;

                let mut hits_for_this_pos = vec![];

                if let Some(c) = Self::cell_for_x(x, &row) {
                    cell = Some(c);
                } else {
                    hits_for_this_pos = y_hits.iter().filter(
                        |hit| hit.x_rng.contains(&x) && hit.y_rng.contains(&y)
                    ).collect::<Vec<&table::RangeHit>>();
                }

                if cell.is_none() && hits_for_this_pos.is_empty() {
                    break;
                }

                let Some(current_date) = Self::cell_for_x(x, &dates) else {
                    continue
                };

                if let Some(cell) = cell {
                    let text = cell.text.replace("\n", " ");
                    let is_just_a_single_number = {
                        text.len() == 1 &&
                        regexes().digit.is_match(&text)
                    };

                    if text.is_empty() || is_just_a_single_number {
                        continue;
                    }

                    let subject = match self.kind {
                        raw::Kind::Groups => parse::subject::groups(
                            &text,
                            num_counter,
                            &cell.color
                        ),
                        raw::Kind::Teachers => todo!()
                    };
                } else if !hits_for_this_pos.is_empty() {

                }
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