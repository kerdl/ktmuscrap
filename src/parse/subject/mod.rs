#[cfg(test)]
mod tests;

use crate::{regexes, options};
use crate::data::schedule;
use crate::parse;


pub fn format_from_color(color: &str) -> schedule::raw::Format {
    match color {
        c if c == &options().settings.parsing.fulltime_color => {
            schedule::raw::Format::Fulltime
        },
        c if c == &options().settings.parsing.remote_color => {
            schedule::raw::Format::Remote
        },
        _ => schedule::raw::Format::Unknown
    }
}

pub fn groups(string: &str, num: u32, color: &str) -> schedule::Subject {
    let raw = string.to_string();
    let format = format_from_color(color);
    let name;
    let attenders;
    let cabinet;

    if let Some((range, parsed)) = parse::attender::teachers(string) {
        let trimmed_name = (&string[..range.start]).trim();
        name = regexes()
            .end_attender_sep
            .replace(trimmed_name, "")
            .parse::<String>()
            .unwrap();
        attenders = parsed;
        cabinet = schedule::Cabinet::default();
    } else {
        if let Some(cabinet_match) = parse::cabinet::from_end(string) {
            cabinet = schedule::Cabinet {
                primary: Some(cabinet_match.as_str().to_string()),
                opposite: None
            };
            name = (&string[..cabinet_match.start()]).trim().to_string();
        } else {
            cabinet = schedule::Cabinet::default();
            name = string.trim().to_string();
        }
        attenders = vec![];
    }

    schedule::Subject {
        raw,
        name,
        num,
        format,
        attenders,
        cabinet
    }
}

pub fn teachers(string: &str, num: u32, color: &str) -> schedule::Subject {
    let raw = string.to_string();
    let format = format_from_color(color);
    let mut name;
    let attenders;
    let cabinet;

    if let Some((range, parsed)) = parse::group::multi(string) {
        let att_raw = (&string[range.start..range.end]).to_string();
        let att_kind = schedule::attender::Kind::Group;
        let att_cabinet = schedule::Cabinet::default();
        name = (&string[..range.start]).to_string();
        attenders = parsed
            .into_iter()
            .map(|att_name| schedule::Attender {
                raw: att_raw.clone(),
                kind: att_kind.clone(),
                name: att_name,
                cabinet: att_cabinet.clone()
            })
            .collect::<Vec<schedule::Attender>>();
    } else {
        name = string.to_string();
        attenders = vec![];
    }

    if let Some(cabinet_match) = parse::cabinet::from_end(&name) {
        cabinet = schedule::Cabinet {
            primary: Some(cabinet_match.as_str().to_string()),
            opposite: None
        };
        name = (&string[..cabinet_match.start()]).to_string();
    } else {
        cabinet = schedule::Cabinet::default();
    }

    schedule::Subject {
        raw,
        name,
        num,
        format,
        attenders,
        cabinet
    }
}
