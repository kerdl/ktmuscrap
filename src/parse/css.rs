use std::collections::HashMap;
use crate::data;


#[derive(PartialEq)]
enum Lookup {
    Key,
    Colon,
    Value
}


pub struct Properties;
impl<'i: 't, 't> Properties {
    pub fn hashmap(parser: &mut cssparser::Parser<'i, 't>) -> data::css::PropsMap<'i> {
        let mut hm = HashMap::new();
        let mut key = cssparser::CowRcStr::default();
        let mut values = vec![];
        let mut stage = Lookup::Key;

        while let Ok(token) = parser.next() {
            match token {
                cssparser::Token::Ident(ident) => {
                    match stage {
                        Lookup::Key => {
                            key = ident.clone();
                            stage = Lookup::Colon;
                        },
                        Lookup::Value => {
                            values.push(data::css::Value::Ident(ident.clone()));
                        },
                        _ => ()
                    }
                },
                cssparser::Token::Colon => {
                    if stage != Lookup::Colon { stage = Lookup::Key; continue; }
                    stage = Lookup::Value;
                },
                cssparser::Token::Hash(hash) => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::Hash(hash.clone()));
                },
                cssparser::Token::IDHash(hash) => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::IDHash(hash.clone()));
                },
                cssparser::Token::QuotedString(string) => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::QuotedString(string.clone()));
                },
                cssparser::Token::UnquotedUrl(url) => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::UnquotedUrl(url.clone()));
                },
                cssparser::Token::Number {
                    has_sign,
                    value,
                    int_value
                } => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::Number(data::css::Number {
                        has_sign: *has_sign,
                        value: *value,
                        int_value: *int_value
                    }));
                },
                cssparser::Token::Percentage {
                    has_sign,
                    unit_value,
                    int_value
                } => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::Percentage(data::css::Percentage {
                        has_sign: *has_sign,
                        unit_value: *unit_value,
                        int_value: *int_value
                    }));
                },
                cssparser::Token::Dimension {
                    has_sign,
                    value,
                    int_value,
                    unit
                } => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    let dim = data::css::UnitNumber {
                        has_sign: *has_sign,
                        value: *value,
                        int_value: *int_value,
                        unit: unit.clone()
                    };
                    values.push(data::css::Value::Dimension(dim));
                },
                cssparser::Token::Function(func) => {
                    if stage != Lookup::Value { stage = Lookup::Key; continue; }
                    values.push(data::css::Value::Function(func.clone()));
                },
                cssparser::Token::Semicolon | cssparser::Token::CloseCurlyBracket => {
                    hm.insert(std::mem::take(&mut key), std::mem::take(&mut values));
                    stage = Lookup::Key;
                },
                _ => ()
            }
        }

        if !hm.contains_key(&key) && !values.is_empty() {
            hm.insert(std::mem::take(&mut key), std::mem::take(&mut values));
        }
    
        hm
    }
}

pub struct Sheet;
impl<'i: 't, 't> Sheet {
    pub fn selectors(parser: &mut cssparser::Parser<'i, 't>) -> data::css::SelectorVec<'i> {
        let mut sels = vec![];
        let mut queries = vec![];
        let mut awaiting_class = false;

        while let Ok(token) = parser.next() {
            match token {
                cssparser::Token::Delim(delim) => {
                    if *delim == '.' {
                        awaiting_class = true;
                    }
                },
                cssparser::Token::Ident(ident) => {
                    let query = if awaiting_class {
                        data::css::Query::Class(ident.clone())
                    } else {
                        data::css::Query::Element(ident.clone())
                    };
                    queries.push(query);
                    awaiting_class = false;
                },
                cssparser::Token::CurlyBracketBlock => {
                    let result = parser.parse_nested_block(|mut parser| {
                        Ok::<data::css::PropsMap, cssparser::ParseError<()>>(
                            Properties::hashmap(&mut parser)
                        )
                    });
                    if let Ok(hm) = result  {
                        let selector = data::css::Selector {
                            query: std::mem::take(&mut queries),
                            properties: hm
                        };
                        sels.push(selector);
                    } else if let Err(_) = result {
                        queries.clear();
                        awaiting_class = false;
                    };
                },
                _ => {
                    awaiting_class = false;
                }
            }
        }

        sels
    }
}

pub fn get_key_from_class<'a>(
    key: &str,
    class: &str,
    styles: &'a data::css::SelectorVec,
) -> Option<&'a Vec<data::css::Value<'a>>> {
    for selector in styles.iter() {
        let is_match = selector.query.iter().find(
            |q| match q {
                data::css::Query::Class(cls) => *cls == class,
                _ => false 
            }).is_some();
        if !is_match {
            continue;
        }
        return selector.properties.get(key);
    }

    None
}

pub fn get_key_from_classes<'a>(
    key: &str,
    classes: &[String],
    styles: &'a data::css::SelectorVec,
) -> Option<&'a Vec<data::css::Value<'a>>> {
    for class in classes.iter() {
        if let Some(value) = get_key_from_class(key, class, styles) {
            return Some(value);
        }
    }

    None
}

pub fn get_any_hash_in_value<'a>(
    values: &'a Vec<data::css::Value<'a>>
) -> Option<&'a cssparser::CowRcStr<'a>> {
    for value in values {
        match value {
            data::css::Value::Hash(hash) => return Some(hash),
            data::css::Value::IDHash(hash) => return Some(hash),
            _ => ()
        }
    }

    None
}