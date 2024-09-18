use serde_derive::{Serialize, Deserialize};


/// # Kind of subject attender
#[derive(
    Serialize, 
    Deserialize, 
    Debug, 
    Clone,
    PartialEq,
    Eq,
    Hash
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Teacher,
    Group,
    Vacancy
}
