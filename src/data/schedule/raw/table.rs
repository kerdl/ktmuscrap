#[derive(Debug, Clone)]
pub struct Cell {
    /// # How wide the cell is
    pub colspan: u32,
    /// # How tall the cell is
    pub rowspan: u32,
    pub text: String,
}
impl Cell {
    pub fn new(
        colspan: u32,
        rowspan: u32,
        text: String,
    ) -> Cell {

        Cell {
            colspan,
            rowspan,
            text
        }
    }
}


#[derive(Debug, Clone)]
pub struct Body {
    pub schema: Vec<Vec<Cell>>,
}
impl Body {
    pub fn new(schema: Vec<Vec<Cell>>, ) -> Body {
        Body { schema }
    }
}