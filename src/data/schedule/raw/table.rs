use std::ops::Range;
use chrono::NaiveDate;
use crate::data::schedule::attender;


pub trait XCord {
    fn x(&self) -> usize;
}

pub trait YCord {
    fn y(&self) -> usize;
}

pub trait Width {
    fn width(&self) -> usize;
}

pub trait Height {
    fn height(&self) -> usize;
}

pub trait XRange {
    fn x_range(&self) -> Range<usize>;
}

pub trait YRange {
    fn y_range(&self) -> Range<usize>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub colspan: usize,
    pub rowspan: usize,
    pub text: String,
    pub color: String
}
impl XCord for Cell {
    fn x(&self) -> usize {
        self.x
    }
}
impl YCord for Cell {
    fn y(&self) -> usize {
        self.y
    }
}
impl Width for Cell {
    fn width(&self) -> usize {
        if self.colspan < 1 { 1 }
        else { self.colspan }
    }
}
impl Height for Cell {
    fn height(&self) -> usize {
        if self.rowspan < 1 { 1 }
        else { self.rowspan }
    }
}
impl XRange for Cell {
    fn x_range(&self) -> Range<usize> {
        self.x()..(self.x() + self.width() - 1)
    }
}
impl YRange for Cell {
    fn y_range(&self) -> Range<usize> {
        self.y()..(self.y() + self.height() - 1)
    }
}
impl Cell {
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
    pub fn does_hit_next_rows(&self) -> bool {
        self.rowspan > 1
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
    pub fn row_hits(&self) -> usize {
        if self.rowspan < 1 { 0 }
        else { self.rowspan - 1 }
    }
}

#[derive(Debug, Clone)]
pub struct OptDate<'a> {
    pub raw: &'a str,
    pub parsed: Option<Range<NaiveDate>>,
    pub range: Range<usize>
}
impl<'a> OptDate<'a> {
    pub fn to_date(self) -> Option<Date<'a>> {
        let Some(parsed) = self.parsed else { return None };
        let date = Date {
            raw: self.raw,
            parsed,
            range: self.range,
        };
        Some(date)
    }
}

#[derive(Debug, Clone)]
pub struct Date<'a> {
    pub raw: &'a str,
    pub parsed: Range<NaiveDate>,
    pub range: Range<usize>
}
impl<'a> XCord for Date<'a> {
    fn x(&self) -> usize {
        self.range.start
    }
}
impl<'a> XRange for Date<'a> {
    fn x_range(&self) -> Range<usize> {
        self.range.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Formation<'a> {
    pub kind: attender::Kind,
    pub raw: &'a str,
    pub valid: String,
    pub range: Range<usize>
}
impl<'a> YRange for Formation<'a> {
    fn y_range(&self) -> Range<usize> {
        self.range.clone()
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