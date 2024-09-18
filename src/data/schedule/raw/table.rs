use std::ops::{Range, RangeInclusive};
use chrono::NaiveDate;
use crate::data::schedule;


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

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub colspan: usize,
    pub rowspan: usize,
    pub text: String,
    pub color: palette::Srgb
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
        self.x()..(self.x() + self.width())
    }
}
impl YRange for Cell {
    fn y_range(&self) -> Range<usize> {
        self.y()..(self.y() + self.height())
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
    pub parsed: Option<RangeInclusive<NaiveDate>>,
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
    pub parsed: RangeInclusive<NaiveDate>,
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
pub struct Formation {
    pub range: Range<usize>,
    pub object: schedule::Formation
}
impl YRange for Formation {
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
pub struct RangeHit<'a> {
    pub by: &'a Cell,
    pub is_done: bool,
}
impl<'a> RangeHit<'a> {
    pub fn done(&mut self) {
        self.is_done = true;
    }
}