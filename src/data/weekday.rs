use ngrammatic::{Corpus, CorpusBuilder, Pad, SearchResult};
use serde_derive::{Serialize, Deserialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumString, EnumIter, Display};
use lazy_static::lazy_static;
use std::str::FromStr;


lazy_static! {
    static ref CORPUS: Corpus = {
        let mut corpus = CorpusBuilder::new()
            .arity(2)
            .pad_full(Pad::Auto)
            .finish();
        
        for weekday in Weekday::iter() {
            corpus.add_text(&weekday.to_string())
        }

        corpus
    };
}


#[derive(Serialize, Deserialize, EnumString, Display, EnumIter, Debug, Clone, PartialEq, Eq)]
pub enum Weekday {
    #[serde(rename = "Понедельник")]
    #[strum(to_string = "Понедельник")]
    Monday,
    #[serde(rename = "Вторник")]
    #[strum(to_string = "Вторник")]
    Tuesday,
    #[serde(rename = "Среда")]
    #[strum(to_string = "Среда")]
    Wednesday,
    #[serde(rename = "Четверг")]
    #[strum(to_string = "Четверг")]
    Thursday,
    #[serde(rename = "Пятница")]
    #[strum(to_string = "Пятница")]
    Friday,
    #[serde(rename = "Суббота")]
    #[strum(to_string = "Суббота")]
    Saturday,
    #[serde(rename = "Воскресенье")]
    #[strum(to_string = "Воскресенье")]
    Sunday
}
impl Weekday {
    pub fn guess(weekday: &str) -> Option<Weekday> {
        CORPUS.search(weekday, 0.5).first().map(
            |weekday: &SearchResult| {
                Weekday::from_str(&weekday.text).ok()
            }
        ).flatten()
    }
}
