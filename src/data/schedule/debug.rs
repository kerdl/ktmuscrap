use chrono::Utc;

use super::{Page, Type};


pub trait Dummy {
    fn dummy() -> Self;
}

impl Dummy for Page {
    fn dummy() -> Self {
        Page {
            raw: "shit".to_owned(),
            raw_types: vec![],
            sc_type: Type::Daily,
            date: Utc::now().date_naive()..Utc::now().date_naive(),
            groups: vec![]
        }
    }
}