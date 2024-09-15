#[cfg(test)]
mod tests;

use std::ops::Range;
use crate::regexes;


pub fn validate(string: &str) -> Option<String> {
    let matched = regexes().group.find(string)?.as_str();
    let without_punctiation = regexes().nonword.replace_all(matched, "").to_string();
    let mut capitalized = without_punctiation.to_uppercase();
    if capitalized.chars().nth(1).unwrap() != 'К' {
        capitalized.insert(1, 'К');
    }
    Some(capitalized)
}

pub fn multi(mut string: &str) -> Option<(Range<usize>, Vec<String>)> {
    let mut output = vec![];

    let start: usize = 0;
    let mut end: usize = 0;

    loop {
        if let Some(group_match) = regexes().start_group.find(&string) {
            end += group_match.end();
            let valid = validate(group_match.as_str()).unwrap();
            output.push(valid);
            string = &string[group_match.end()..];
        } else if let Some(sep_match) = regexes().start_attender_sep.find(&string) {
            if output.is_empty() { return None }
            end += sep_match.end();
            string = &string[sep_match.end()..];
        } else if let Some(num_match) = regexes().start_digits.find(&string) {
            if output.is_empty() { return None }
            end += num_match.end();
            let last_group = output.last().unwrap();
            let mut last_with_current_num = regexes()
                .end_digits
                .replace(&last_group, "")
                .parse::<String>()
                .unwrap();
            last_with_current_num.push_str(num_match.as_str());
            output.push(last_with_current_num);
            string = &string[num_match.end()..];
        } else {
            break;
        }
    }

    Some((start..end, output))
}
