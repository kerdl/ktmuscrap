#[cfg(test)]
mod tests;

use crate::regexes;


pub fn validate_tokens(tokens: &[&str]) -> String {
    let mut output = "".to_string();

    if tokens.len() == 2 {
        output.push_str(tokens.get(0).unwrap());
        output.push_str(" ");
        output.push_str(tokens.get(1).unwrap());
        output.push_str(".");
    } else if tokens.len() == 3 {
        output.push_str(tokens.get(0).unwrap());
        output.push_str(" ");
        output.push_str(tokens.get(1).unwrap());
        output.push_str(".");
        output.push_str(tokens.get(2).unwrap());
        output.push_str(".");
    }

    output
}

pub fn validate(string: &str) -> Option<String> {
    let matched = regexes()
        .teacher
        .find(string)?
        .as_str();
    let tokens = regexes()
        .nonword
        .split(matched)
        .filter(|tok| !tok.is_empty())
        .collect::<Vec<&str>>();

    if tokens.len() < 2 || tokens.len() > 3 {
        return None;
    }

    Some(validate_tokens(&tokens))
}

pub fn validate_all(string: &str) -> Vec<String> {
    let mut output = vec![];

    let matched = regexes()
        .teacher
        .find_iter(string);

    for m in matched {
        let tokens = regexes()
            .nonword
            .split(m.as_str())
            .filter(|tok| !tok.is_empty())
            .collect::<Vec<&str>>();

        if tokens.len() < 2 || tokens.len() > 3 {
            continue;
        }

        let valid = validate_tokens(&tokens);
        output.push(valid);
    }

    output
}