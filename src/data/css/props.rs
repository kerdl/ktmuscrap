#[derive(Debug, Clone)]
pub struct Number {
    pub has_sign: bool,
    pub value: f32,
    pub int_value: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Percentage {
    pub has_sign: bool,
    pub unit_value: f32,
    pub int_value: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct UnitNumber<'a> {
    pub has_sign: bool,
    pub value: f32,
    pub int_value: Option<i32>,
    pub unit: cssparser::CowRcStr<'a>
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Ident(cssparser::CowRcStr<'a>),
    Hash(cssparser::CowRcStr<'a>),
    IDHash(cssparser::CowRcStr<'a>),
    QuotedString(cssparser::CowRcStr<'a>),
    UnquotedUrl(cssparser::CowRcStr<'a>),
    Number(Number),
    Percentage(Percentage),
    Dimension(UnitNumber<'a>),
    Function(cssparser::CowRcStr<'a>),
}
