use crate::REGEX;


/// # Parse number from string that contains digits and something else
pub fn parse_from_digit_containig(string: &str) -> Option<u32> {
    REGEX.digit.find(string)?.as_str().parse().ok()
}

/// # Parse number from string that contains only digits
pub fn parse_from_digits_only(string: &str) -> Option<u32> {
    string.parse().ok()
}