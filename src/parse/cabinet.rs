use crate::regexes;


pub fn from_end<'a>(string: &'a str) -> Option<regex::Match<'a>> {
    regexes().end_cabinet.find(string)
}