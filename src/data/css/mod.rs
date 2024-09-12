pub mod query;
pub mod props;

use std::collections::HashMap;
pub use query::Query;
pub use props::{
    Number,
    Percentage,
    UnitNumber,
    Value
};


pub type PropsMap<'i> = HashMap<cssparser::CowRcStr<'i>, Value<'i>>;
pub type SelectorVec<'a> = Vec<Selector<'a>>;


#[derive(Debug, Clone)]
pub struct Selector<'a> {
    pub query: Vec<Query<'a>>,
    pub properties: HashMap<String, Value<'a>>
}

#[derive(Debug, Clone)]
pub struct Sheet<'a> {
    pub selectors: Vec<Selector<'a>>
}
