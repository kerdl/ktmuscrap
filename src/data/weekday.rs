use ngrammatic::{Corpus, CorpusBuilder, Pad, SearchResult};
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


#[derive(EnumString, Display, EnumIter, Debug, Clone)]
pub enum Weekday {
    #[strum(to_string = "Понедельник")]
    Monday,
    #[strum(to_string = "Вторник")]
    Tuesday,
    #[strum(to_string = "Среда")]
    Wednesday,
    #[strum(to_string = "Четверг")]
    Thursday,
    #[strum(to_string = "Пятница")]
    Friday,
    #[strum(to_string = "Суббота")]
    Saturday,
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
