use serde::Serialize;
use std::sync::Arc;

use crate::compare::schedule::{Page, TchrPage};
use super::{update::Invoker};


#[derive(Debug, Clone, Serialize)]
pub struct Notify {
    pub random: String,
    pub invoker: Invoker,
    pub daily: Option<Page>,
    pub weekly: Option<Page>,
    pub tchr_daily: Option<TchrPage>,
    pub tchr_weekly: Option<TchrPage>
}