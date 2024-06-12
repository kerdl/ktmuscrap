use derive_new::new;


#[derive(new, Debug, Clone, PartialEq, Eq)]
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
    pub fn width(&self) -> usize {
        if self.colspan < 1 { 1 }
        else { self.colspan }
    }

    pub fn height(&self) -> usize {
        if self.rowspan < 1 { 1 }
        else { self.rowspan }
    }

    /// # If this cell spreads to next rows
    ///
    /// ```notrust
    /// ■ □ □
    /// □ □ □ -> false
    /// □ □ □
    /// 
    /// █ □ □
    /// █ □ □ -> true
    /// □ □ □
    /// ```
    pub fn hits_next_rows(&self) -> bool {
        self.rowspan > 0
    }

    /// # How much other rows this cell affects
    /// (not including its original row)
    /// 
    /// ```notrust
    /// ■ □ □
    /// □ □ □ -> 0 rows
    /// □ □ □
    /// 
    /// █ □ □
    /// █ □ □ -> 2 rows
    /// █ □ □
    /// ```
    pub fn hits(&self) -> usize {
        if self.rowspan < 1 { 0 }
        else { self.rowspan - 1 }
    }
}

/// # `X` axis jumping conditions
#[derive(Debug, Clone)]
pub struct XJump {
    /// # Should jump on this X coordinate
    pub at_x: usize,
    /// # Should jump on this Y coordinate
    pub at_y: usize,
    /// # Should jump X by this amount
    /// (`x += xjump.by`)
    pub by: usize,
    /// # If this jump was performed
    pub is_done: bool,
}
impl XJump {
    pub fn done(&mut self) {
        self.is_done = true;
    }
}

#[derive(Debug, Clone)]
pub struct Hit {
    pub by: Cell,
    pub at_x: usize,
    pub at_y: usize,
    pub is_done: bool,
}
impl Hit {
    pub fn done(&mut self) {
        self.is_done = true;
    }
}

#[derive(Debug, Clone)]
pub struct RangeHit {
    pub by: Cell,
    pub x_rng: std::ops::Range<usize>,
    pub y_rng: std::ops::Range<usize>,
    pub is_done: bool,
}
impl RangeHit {
    pub fn done(&mut self) {
        self.is_done = true;
    }
}

#[derive(new, Debug, Clone)]
pub struct Body {
    pub schema: Vec<Vec<Cell>>,
}

#[derive(new, Debug, Clone)]
pub struct Group {
    pub raw: String,
    pub valid: String,
}

#[derive(new, Debug, Clone, PartialEq, Eq)]
pub struct Teacher {
    pub raw: String,
    pub valid: String,
}