use crate::data::schedule::raw::table;


#[derive(thiserror::Error, Debug)]
#[error("table schema parsing error")]
pub enum ParsingError {

}


pub struct Parser {
    pub schema: Vec<Vec<table::Cell>>
}
impl Parser {
    pub fn from_schema(schema: Vec<Vec<table::Cell>>) -> Self {
        Self { schema }
    }

    pub async fn parse(&self) -> Result<(), ParsingError> {
        todo!();
    }
}