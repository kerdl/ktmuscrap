use itertools::Itertools;
use std::ops::Range;
use crate::regexes;
use crate::data::schedule;
use crate::parse;


const VACANCY: &str = "Вакансия";


pub fn teachers(string: &str) -> Option<(Range<usize>, Vec<schedule::Attender>)> {
    let start;
    let end;

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

    let mut attenders = vec![];

    if let Some(start_pos) = attenders_start_pos {
        start = start_pos;
        end = string.len();

        for (idx, (kind, attender_match)) in consecutive_attender_matches
            .iter()
            .enumerate()
        {
            let next = consecutive_attender_matches.get(idx + 1);
            let related_until = next
                .map(|(_kind, m)| m.start())
                .unwrap_or(string.len());
            let related_text = &string
                [attender_match.end()..related_until];
            let valid_attender = match kind {
                schedule::attender::Kind::Teacher => {
                    parse::teacher::validate(attender_match.as_str()).unwrap()
                },
                schedule::attender::Kind::Vacancy => {
                    attender_match.as_str().to_string()
                },
                _ => unreachable!()
            };

            let mut raw = attender_match.as_str().to_string();
            raw.push_str(related_text);

            let trimmed_related_text = related_text.trim();
            let trimmed_related_text = regexes()
                .start_attender_sep
                .replace(trimmed_related_text, "")
                .parse::<String>()
                .unwrap();
            let trimmed_related_text = regexes()
                .end_attender_sep
                .replace(&trimmed_related_text, "")
                .parse::<String>()
                .unwrap();

            let primary_cabinet = if trimmed_related_text.is_empty() {
                None
            } else {
                Some(trimmed_related_text)
            };

            let cabinet = schedule::Cabinet {
                primary: primary_cabinet,
                opposite: None
            };

            let attender = schedule::Attender {
                raw,
                kind: kind.clone(),
                name: valid_attender,
                cabinet
            };

            attenders.push(attender);
        }
    } else {
        return None;
    }

    Some((start..end, attenders))
}
