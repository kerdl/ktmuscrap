use crate::REGEX;


pub fn parse(string: &str) -> Option<u32> {
    REGEX.digit.find(string)?.as_str().parse().ok()
}