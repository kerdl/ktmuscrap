#[derive(Debug, Clone)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    /// # How wide the cell is
    pub colspan: usize,
    /// # How tall the cell is
    pub rowspan: usize,
    pub text: String,
}
impl Cell {
    pub fn new(
        x: usize,
        y: usize,
        colspan: usize,
        rowspan: usize,
        text: String,
    ) -> Cell {

        Cell {
            x,
            y,
            colspan,
            rowspan,
            text,
        }
    }
}

/// # `X` axis jumping conditions
#[derive(Debug, Clone)]
pub struct XJump {
    pub at_x: usize,
    pub at_y: usize,
    pub by: usize,
    pub is_done: bool,
}
impl XJump {
    pub fn done(&mut self) {
        self.is_done = true;
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
