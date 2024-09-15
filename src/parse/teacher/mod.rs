#[cfg(test)]
mod tests;

use crate::regexes;


pub fn validate(string: &str) -> Option<String> {
    let mut output = "".to_string();
    let matched = regexes()
        .teacher
        .find(string)?
        .as_str();
    let tokens = regexes()
        .nonword
        .split(matched)
        .filter(|tok| !tok.is_empty())
        .collect::<Vec<&str>>();

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
    } else {
        return None
    }

    Some(output)
}