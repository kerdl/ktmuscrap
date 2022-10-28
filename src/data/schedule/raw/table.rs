#[derive(Debug, Clone)]
pub struct Cell {
    pub x_index: usize,
    pub y_index: usize,
    /// # How wide the cell is
    pub colspan: u32,
    /// # How tall the cell is
    pub rowspan: u32,
    pub text: String,
}
impl Cell {
    pub fn new(
        x_index: usize,
        y_index: usize,
        colspan: u32,
        rowspan: u32,
        text: String,
    ) -> Cell {

        Cell {
            x_index,
            y_index,
            colspan,
            rowspan,
            text
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub y_index: usize,
}
impl Row {
    pub fn new(y_index: usize) -> Row {
        Row { y_index }
    }
}

