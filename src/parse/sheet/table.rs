use chrono::NaiveDate;
use crate::data::schedule::raw::table;
use crate::regexes;


#[derive(thiserror::Error, Debug)]
#[error("table schema parsing error")]
pub enum ParsingError {
    NoDateRow
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

    pub async fn parse(&self) -> Result<(), ParsingError> {
        let Some(dates_row) = self.date_row() else {
            return Err(ParsingError::NoDateRow)
        };

        let mut current_formation: Option<table::Formation> = None;

        for row in self.schema.iter() {
            for cell in row.iter() {

            }
        }

        Ok(())
    }

    pub fn row_for_y(y: usize, schema: &Vec<Vec<table::Cell>>) -> Option<&Vec<table::Cell>> {
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
}