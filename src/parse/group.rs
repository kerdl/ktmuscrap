use crate::REGEX;
use crate::data::schedule::Subgroup;


pub fn parse(string: &str) -> Option<String> {
    let matched = REGEX.group.find(string)?.as_str();
    let without_punctiation = REGEX.nonword.replace_all(matched, "").to_string();
    let capitalized = without_punctiation.to_uppercase();

    Some(capitalized)
}

pub fn extract_and_parse(string: &mut String) -> Option<String> {
    let string_clone = string.clone();
    let matched = REGEX.group.find(string_clone.as_str())?;
    string.replace_range(matched.start()..matched.end(), "");
    let without_punctiation = REGEX.nonword.replace_all(matched.as_str(), "").to_string();
    let capitalized = without_punctiation.to_uppercase();

    Some(capitalized)
}

pub fn parse_multiple(string: &str) -> Vec<Subgroup> {
    let mut parsed = vec![];
    let raw_groups = string.split(",").map(|group| group.trim().to_string());

    for mut raw_group in raw_groups {
        let Some(group) = extract_and_parse(&mut raw_group) else {
            continue;
        };

        let rest = raw_group.trim();

        let parsed_group = Subgroup {
            group,
            subgroup: if !rest.is_empty() {
                Some(rest.to_string())
            } else {
                None
            },
        };
        parsed.push(parsed_group)
    }

    parsed
}

pub fn extract_from_start(string: &mut String) -> Vec<String> {
    let mut groups = vec![];

    // while we can find a group from THE START of the string
    while let Some(group) = REGEX.group.find(&string) {
        // push this group match to vec
        groups.push(group.as_str().to_owned());

        // from start of the string, remove this group
        *string = REGEX.group.replace(string, "").to_string();

        // if there's another group left
        if REGEX.group.is_match(&string) {
            // remove first characters from the string
            // until it hits a number
            while REGEX.start_group.find(&string).is_none() {
                if string.len() > 0 {
                    string.remove(0); // remove first
                }
            }
        }

        // remove whitespaces from the beginning and end
        *string = string.trim().to_string();
    }

    groups.into_iter().rev().collect()
}