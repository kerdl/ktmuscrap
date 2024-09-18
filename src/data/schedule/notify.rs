use serde::Serialize;
use crate::compare::schedule::Page;


#[derive(Debug, Clone, Serialize)]
pub struct Notify {
    pub random: String,
    pub groups: Option<Page>,
    pub teachers: Option<Page>
}