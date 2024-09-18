use std::collections::HashMap;
use std::borrow::Cow;
use cssparser::CowRcStr;
use crate::data::css::Value;
use super::*;


#[test]
fn test_properties_1() {
    let sample = "color: rgb(255, 0, 255);";
    let mut input = cssparser::ParserInput::new(sample);
    let mut parser = cssparser::Parser::new(&mut input);
    let result = Properties::hashmap(&mut parser);
    let mut expected = HashMap::new();
    expected.insert(
        CowRcStr::from(Cow::Borrowed("color")),
        vec![Value::Function("rgb(255, 0, 255)".to_string())]
    );
    
    assert_eq!(result, expected);
}