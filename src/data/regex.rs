use regex::Regex;
use serde_derive::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::{sync::Arc, collections::HashSet};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Group,
    Date,
    Time,
    Teacher,
    Cabinet
}
impl Type {
    pub fn to_str(&self) -> &'static str {
        match self {
            Type::Group   => { "group" }
            Type::Date    => { "date" }
            Type::Time    => { "time" }
            Type::Teacher => { "teacher" }
            Type::Cabinet => { "cabinet" }
        }
    }
}

pub struct Container {
    /// ## Matches
    /// - **"1КДД69"**
    /// - **"1-кДД-69"**
    /// - ...
    pub group: Arc<Regex>,
    /// ## Matches
    /// - **"01.01.22"**
    /// - **"01/01/22"**
    /// - ...
    /// 
    /// ( *day*.*month*.*year* )
    pub date: Arc<Regex>,
    /// ## Matches
    /// - **"8:00-9:00"**
    /// - **"10:00-11:00"**
    /// - ...
    pub time: Arc<Regex>,
    /// ## Matches
    /// - **"Ебанько Х."**
    /// - **"Ебанько Х.Й"**
    /// - **"Ебанько Х.Й."**
    /// - ...
    pub teacher: Arc<Regex>,
    /// ## Matches
    /// - **"ауд.29"**
    /// - **"ауд.56,54"**
    /// - **"ауд.сп.з,23в"**
    /// - ...
    pub cabinet: Arc<Regex>,
}
impl Default for Container {
    fn default() -> Container {
        let group   = r"(\d)([-]{0,1})([а-яёА-ЯЁ]{3})([-]{0,1})(\d{2})";
        let date    = r"(\d{1,2})\W(\d{1,2})\W(\d{2})";
        let time    = r"(\d{1,2}:\d{2})-(\d{1,2}:\d{2})";
        let teacher = r"([А-ЯЁ][а-яё]{1,})(\s)([А-ЯЁ]{1}[.])([А-ЯЁ]{1}[.]{0,1}){0,1}";
        let cabinet = r"([аa][уy]д)[.].*(\d|\w)([.]|[,]){0,1}";

        Container {
            group: Arc::new(Regex::new(group).unwrap()), 
            date: Arc::new(Regex::new(date).unwrap()), 
            time: Arc::new(Regex::new(time).unwrap()), 
            teacher: Arc::new(Regex::new(teacher).unwrap()), 
            cabinet: Arc::new(Regex::new(cabinet).unwrap())
        }
    }
}