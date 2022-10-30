use crate::REGEX;


pub fn parse(string: &str) -> Option<String> {
    let matched = REGEX.group.find(string)?.as_str();
    let without_punctiation = REGEX.nonword.replace_all(matched, "").to_string();
    let capitalized = without_punctiation.to_uppercase();

    Some(capitalized)
}