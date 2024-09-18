#[cfg(test)]
mod tests;

use palette::IntoColor;
use palette::color_difference::Ciede2000;
use crate::{regexes, options};
use crate::data::schedule;
use crate::parse;


pub fn format_from_color(color: palette::Srgb) -> schedule::raw::Format {
    if color.red == 255.0 && color.green == 255.0 && color.blue == 255.0 {
        return schedule::raw::Format::Unknown;
    }

    let lab: palette::Lab = color.into_color();
    let fulltime_diff = lab.difference(
        options().settings.parsing.fulltime_lab
    );
    let remote_diff = lab.difference(
        options().settings.parsing.remote_lab
    );

    if fulltime_diff > 45.0 && remote_diff > 45.0 {
        return schedule::raw::Format::Unknown;
    }

    if fulltime_diff < remote_diff {
        // if the cell is #fce5cd (exact match)
        // fulltime_diff == 33.8717690
        // remote_diff == 45.0357895
        schedule::raw::Format::Fulltime
    } else {
        // if the cell is #c6d9f0 (exact match)
        // fulltime_diff == 44.7135468
        // remote_diff == 20.7941818
        schedule::raw::Format::Remote
    }
}

pub fn groups(string: &str, num: u32, color: palette::Srgb) -> schedule::Subject {
    let raw = string.to_string();
    let format = format_from_color(color);
    let name;
    let attenders;

    if let Some((range, parsed)) = parse::attender::teachers(string) {
        let trimmed_name = (&string[..range.start]).trim();
        name = regexes()
            .end_attender_sep
            .replace(trimmed_name, "")
            .parse::<String>()
            .unwrap();
        attenders = parsed;
    } else {
        name = string.trim().to_string();
        attenders = vec![];
    }

    schedule::Subject {
        raw,
        name,
        num,
        format,
        attenders
    }
}

pub fn teachers(string: &str, num: u32, color: palette::Srgb) -> schedule::Subject {
    let raw = string.to_string();
    let format = format_from_color(color);
    let mut name;
    let mut attenders;

    if let Some(parsed) = parse::group::multi(string) {
        let att_kind = schedule::attender::Kind::Group;
        let att_cabinet = schedule::Cabinet::default();
        name = (&string[parsed.last().unwrap().0.end..]).trim().to_string();
        attenders = parsed
            .into_iter()
            .map(|att| schedule::Attender {
                raw: (&string[att.0.start..att.0.end]).to_string(),
                kind: att_kind.clone(),
                name: att.1,
                cabinet: att_cabinet.clone()
            })
            .collect::<Vec<schedule::Attender>>();
    } else {
        name = string.to_string();
        attenders = vec![];
    }

    if let Some(cabinet_match) = parse::cabinet::from_end(&name) {
        let cab = schedule::Cabinet {
            primary: Some(cabinet_match.as_str().to_string()),
            opposite: None
        };
        for att in attenders.iter_mut() {
            att.cabinet = cab.clone();
        }
        name = (&name[..cabinet_match.start()]).trim().to_string();
    }

    schedule::Subject {
        raw,
        name,
        num,
        format,
        attenders
    }
}
