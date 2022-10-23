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
    pub group: Arc<RwLock<Option<Regex>>>,
    pub date: Arc<RwLock<Option<Regex>>>,
    pub time: Arc<RwLock<Option<Regex>>>,
    pub teacher: Arc<RwLock<Option<Regex>>>,
    pub cabinet: Arc<RwLock<Option<Regex>>>,
}
impl Container {
    pub fn new(
        group: Arc<RwLock<Option<Regex>>>,
        date: Arc<RwLock<Option<Regex>>>,
        time: Arc<RwLock<Option<Regex>>>,
        teacher: Arc<RwLock<Option<Regex>>>,
        cabinet: Arc<RwLock<Option<Regex>>>,
    ) -> Container {
        Container { 
            group, 
            date, 
            time, 
            teacher, 
            cabinet 
        }
    }

    pub async fn unset_types(self: Arc<Self>) -> HashSet<Type> {
        let mut types: HashSet<Type> = HashSet::new();

        // is there really no way to iterate
        // struct fields?

        if self.group.read().await.is_none() {
            types.insert(Type::Group);
        }

        if self.date.read().await.is_none() {
            types.insert(Type::Date);
        }

        if self.time.read().await.is_none() {
            types.insert(Type::Time);
        }

        if self.teacher.read().await.is_none() {
            types.insert(Type::Teacher);
        }

        if self.cabinet.read().await.is_none() {
            types.insert(Type::Cabinet);
        }

        types
    }
}
impl Default for Container {
    fn default() -> Container {
        Container::new(
            Arc::new(RwLock::new(None)), 
            Arc::new(RwLock::new(None)), 
            Arc::new(RwLock::new(None)), 
            Arc::new(RwLock::new(None)), 
            Arc::new(RwLock::new(None))
        )
    }
}