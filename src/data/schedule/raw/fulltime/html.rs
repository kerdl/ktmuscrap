use derive_new::new;
use crate::data::schedule::raw::table::Cell;


#[derive(new, Debug, Clone)]
pub struct HeaderTable {
    pub header: String,
    pub table: Vec<Vec<String>>,
}

#[derive(new, Debug, Clone)]
pub struct HeaderSpanTable {
    pub header: String,
    pub table: Vec<Vec<Cell>>,
}
