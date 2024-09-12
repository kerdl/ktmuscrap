use std::collections::HashMap;
use crate::data;


#[derive(PartialEq)]
enum Lookup {
    Ident,
    Colon,
    Value
}


pub struct Properties<'i> {
    input: cssparser::ParserInput<'i>
}
impl<'i> Properties<'i> {
    pub fn from_str(string: &'i str) -> Self {
        Self {
            input: cssparser::ParserInput::new(string)
        }
    }

    pub fn hashmap(&mut self) -> data::css::PropsMap {
        let mut hm = HashMap::new();

        let mut parser = cssparser::Parser::new(&mut self.input);
        let mut key = cssparser::CowRcStr::default();
        let mut stage = Lookup::Ident;

        while let Ok(token) = parser.next() {
            match token {
                cssparser::Token::Ident(ident) => {
                    key = ident.clone();
                    stage = Lookup::Colon;
                },
                cssparser::Token::Colon => {
                    if stage != Lookup::Colon { stage = Lookup::Ident; continue; }
                    stage = Lookup::Value;
                },
                cssparser::Token::IDHash(hash) => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    hm.insert(key.clone(), data::css::Value::IDHash(hash.clone()));
                    stage = Lookup::Ident;
                },
                cssparser::Token::QuotedString(string) => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    hm.insert(key.clone(), data::css::Value::QuotedString(string.clone()));
                    stage = Lookup::Ident;
                },
                cssparser::Token::UnquotedUrl(url) => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    hm.insert(key.clone(), data::css::Value::UnquotedUrl(url.clone()));
                    stage = Lookup::Ident;
                },
                cssparser::Token::Number {
                    has_sign,
                    value,
                    int_value
                } => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    hm.insert(key.clone(), data::css::Value::Number(data::css::Number {
                        has_sign: *has_sign,
                        value: *value,
                        int_value: *int_value
                    }));
                    stage = Lookup::Ident;
                },
                cssparser::Token::Percentage {
                    has_sign,
                    unit_value,
                    int_value
                } => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    hm.insert(key.clone(), data::css::Value::Percentage(data::css::Percentage {
                        has_sign: *has_sign,
                        unit_value: *unit_value,
                        int_value: *int_value
                    }));
                    stage = Lookup::Ident;
                },
                cssparser::Token::Dimension {
                    has_sign,
                    value,
                    int_value,
                    unit
                } => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    let dim = data::css::UnitNumber {
                        has_sign: *has_sign,
                        value: *value,
                        int_value: *int_value,
                        unit: unit.clone()
                    };
                    hm.insert(key.clone(), data::css::Value::Dimension(dim));
                    stage = Lookup::Ident;
                },
                cssparser::Token::Function(func) => {
                    if stage != Lookup::Value { stage = Lookup::Ident; continue; }
                    hm.insert(key.clone(), data::css::Value::Function(func.clone()));
                    stage = Lookup::Ident;
                },
                _ => { stage = Lookup::Ident; continue; }
            }
        }
    
        hm
    }
}

pub struct Sheet<'i> {
    input: cssparser::ParserInput<'i>
}
impl<'i> Sheet<'i> {
    pub fn from_str(string: &'i str) -> Self {
        Self {
            input: cssparser::ParserInput::new(string)
        }
    }

    pub fn selectors(&mut self) -> data::css::SelectorVec {
        let mut sels = vec![];

        let mut parser = cssparser::Parser::new(&mut self.input);
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
                    parser.parse_nested_block(|parser| -> Result<data::css::PropsMap, cssparser::ParseError<()>> {
                        todo!();
                    });
                },
                _ => {
                    awaiting_class = false;
                }
            }
        }

        sels
    }
}