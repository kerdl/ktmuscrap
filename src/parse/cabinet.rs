use crate::REGEX;


pub fn extract_from_end(string: &mut String) -> Option<String> {
    let (whole_name, maybe_cabinet) = string.rsplit_once(" ")?;

    let cabinet = {
        REGEX.cabinet.find(maybe_cabinet)?
        .as_str().to_owned()
    };

    let leftover = REGEX.cabinet.replace(maybe_cabinet, "");
    *string = {
        format!("{} {}", whole_name, leftover.to_string())
        .trim().to_owned()
    };

    Some(cabinet)
}