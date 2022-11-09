use serde::Serialize;

use crate::compare::schedule::Page;


#[derive(Debug, Serialize)]
pub struct Notify {
    pub daily: Option<Page>,
    pub weekly: Option<Page>
}