use chrono::{NaiveDate, Duration, Datelike};
use ngrammatic::{Corpus, CorpusBuilder, Pad, SearchResult};
use serde_derive::{Serialize, Deserialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumString, EnumIter, Display};
use lazy_static::lazy_static;
use std::{str::FromStr, ops::Range, collections::HashMap};


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

    static ref CHRONO_WEEKDAY_MAP: HashMap<chrono::Weekday, Weekday> = {
        let mut map = HashMap::new();

        map.insert(chrono::Weekday::Mon, Weekday::Monday);
        map.insert(chrono::Weekday::Tue, Weekday::Tuesday);
        map.insert(chrono::Weekday::Wed, Weekday::Wednesday);
        map.insert(chrono::Weekday::Thu, Weekday::Thursday);
        map.insert(chrono::Weekday::Fri, Weekday::Friday);
        map.insert(chrono::Weekday::Sat, Weekday::Saturday);
        map.insert(chrono::Weekday::Sun, Weekday::Sunday);

        map
    };
}


#[derive(
    Serialize, 
    Deserialize, 
    EnumString, 
    Display, 
    EnumIter, 
    Debug, 
    Clone, 
    PartialEq, 
    Eq,
    PartialOrd,
    Ord,
    Hash
)]
pub enum Weekday {
    #[serde(rename = "понедельник")]
    #[strum(to_string = "понедельник")]
    Monday,
    #[serde(rename = "вторник")]
    #[strum(to_string = "вторник")]
    Tuesday,
    #[serde(rename = "среда")]
    #[strum(to_string = "среда")]
    Wednesday,
    #[serde(rename = "четверг")]
    #[strum(to_string = "четверг")]
    Thursday,
    #[serde(rename = "пятница")]
    #[strum(to_string = "пятница")]
    Friday,
    #[serde(rename = "суббота")]
    #[strum(to_string = "суббота")]
    Saturday,
    #[serde(rename = "воскресенье")]
    #[strum(to_string = "воскресенье")]
    Sunday
}
impl Weekday {
    pub fn guess(weekday: &str) -> Option<Weekday> {
        CORPUS.search(&weekday.to_lowercase(), 0.5).first().map(
            |weekday: &SearchResult| {
                Weekday::from_str(&weekday.text).ok()
            }
        ).flatten()
    }

    pub fn date_from_range(
        &self,
        range: &Range<NaiveDate>
    ) -> Option<NaiveDate> {

        let mut date = range.start;

        while range.start <= range.end {
            if self == CHRONO_WEEKDAY_MAP.get(&date.weekday()).unwrap() {
                return Some(date)
            }

            date = date + Duration::days(1);
        }

        None
    }
}
