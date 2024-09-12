#[derive(Debug, Clone)]
pub enum Query<'a> {
    Element(cssparser::CowRcStr<'a>),
    Class(cssparser::CowRcStr<'a>),
}