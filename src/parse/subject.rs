use itertools::Itertools;
use crate::{regexes, options};
use crate::data::schedule;


const VACANCY: &str = "Вакансия";


pub fn groups(string: &str, num: u32, color: &str) -> schedule::Subject {
    let raw = string.to_string();
    let format = match color {
        c if c == &options().settings.parsing.fulltime_color => {
            schedule::raw::Format::Fulltime
        },
        c if c == &options().settings.parsing.remote_color => {
            schedule::raw::Format::Remote
        },
        _ => schedule::raw::Format::Unknown
    };

    let teacher_matches = regexes()
        .teacher
        .find_iter(string)
        .map(|m| (schedule::attender::Kind::Teacher, m))
        .collect::<Vec<(schedule::attender::Kind, regex::Match)>>();
    let vacancy_matches = regexes()
        .vacancy
        .find_iter(string)
        .filter(|vac| strsim::damerau_levenshtein(VACANCY, vac.as_str()) > 2)
        .map(|m| (schedule::attender::Kind::Vacancy, m))
        .collect::<Vec<(schedule::attender::Kind, regex::Match)>>();

    let consecutive_attender_matches = teacher_matches
        .into_iter()
        .chain(vacancy_matches.into_iter())
        .sorted_by(|a, b| Ord::cmp(&a.1.start(), &b.1.start()))
        .collect::<Vec<(schedule::attender::Kind, regex::Match)>>();

    let attenders_start_pos = consecutive_attender_matches
        .get(0)
        .map(|(_kind, m)| m.start());

    let name;
    let mut attenders = vec![];

    if let Some(start_pos) = attenders_start_pos {
        name = (&string[..start_pos]).to_string();
        let attenders_string = &string[start_pos..];

        for (idx, (kind, attender_match)) in consecutive_attender_matches
            .iter()
            .enumerate()
        {
            let next = consecutive_attender_matches.get(idx + 1);
            let related_until = next
                .map(|(_kind, m)| m.start())
                .unwrap_or(attenders_string.len());
            let related_text = &attenders_string
                [attender_match.end()..related_until];

            let mut raw = attender_match.as_str().to_string();
            raw.push_str(related_text);

            let primary_cabinet = related_text.trim().to_string();

            let cabinet = schedule::Cabinet {
                primary: if !primary_cabinet.is_empty() {
                    Some(primary_cabinet)
                } else {
                    None
                },
                opposite: None
            };

            let attender = schedule::Attender {
                raw,
                kind: kind.clone(),
                name: attender_match.as_str().to_string(),
                cabinet
            };

            attenders.push(attender);
        }
    } else {
        // whole string is just a subject name
        name = raw.to_string();
    }

    schedule::Subject {
        raw,
        name,
        num,
        format,
        attenders
    }
}

pub fn teachers(
    string: &str,
    num: u32,
    known_cabinets: Vec<String>,
    color: &str
) -> Option<schedule::Subject> {
    None
}