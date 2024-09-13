use crate::regexes;


pub fn validate(string: &str) -> Option<String> {
    let matched = regexes().group.find(string)?.as_str();
    let without_punctiation = regexes().nonword.replace_all(matched, "").to_string();
    let capitalized = without_punctiation.to_uppercase();
    Some(capitalized)
}