use std::path::PathBuf;
use crate::data::{self, schedule::raw::table};
use crate::parse;

 
const STYLE: &str = "style";
const DIV: &str = "div";
const TABLE: &str = "table";
const TBODY: &str = "tbody";
const TR: &str = "tr";
const TD: &str = "td";
const COLSPAN: &str = "colspan";
const ROWSPAN: &str = "rowspan";
const HEIGHT: &str = "height";
const GRID_CONTAINER: &str = "grid-container";
const FREEZEBAR_CELL: &str = "freezebar-cell";
const BACKGROUND_COLOR: &str = "background-color";
const DEFAULT_CELL_COLOR: &str = "#ffffff";
const ZERO: &str = "0";



#[derive(thiserror::Error, Debug)]
#[error("html parsing error")]
pub enum ParsingError {
    LoadIO(std::io::Error),
    LoadHtmlParser(html_parser::Error),
    NoTbody,
}
impl From<std::io::Error> for ParsingError {
    fn from(value: std::io::Error) -> Self {
        Self::LoadIO(value)
    }
}
impl From<html_parser::Error> for ParsingError {
    fn from(value: html_parser::Error) -> Self {
        Self::LoadHtmlParser(value)
    }
}

pub struct Parser {
    pub string_dom: String,
    pub dom: html_parser::Dom
}
impl Parser {
    pub async fn from_string(string: String) -> Result<Self, html_parser::Error> {
        let dom = parse::dom_from_string(&string).await?;
        let this = Self {
            string_dom: string,
            dom
        };
        Ok(this)
    }

    pub async fn from_path(path: &PathBuf) -> Result<Self, ParsingError> {
        let string = tokio::fs::read_to_string(path).await?;
        let this = Self::from_string(string).await?;
        Ok(this)
    }

    fn style(&self) -> Option<&html_parser::Node> {
        self.dom.children.iter().find(|node| {
            let Some(elm) = node.element() else { return false };
            elm.name == STYLE
        })
    }

    fn main_div(&self) -> Option<&html_parser::Node> {
        self.dom.children.iter().find(|node| {
            let Some(elm) = node.element() else { return false };
            let is_div = elm.name == DIV;
            let is_grid_container = elm.classes.join(" ").contains(GRID_CONTAINER);
            is_div && is_grid_container
        })
    }

    fn main_table(&self) -> Option<&html_parser::Node> {
        let Some(node) = self.main_div() else { return None };
        let Some(elm) = node.element() else { return None };
        elm.children.iter().find(|node| {
            let Some(elm) = node.element() else { return false };
            elm.name == TABLE
        })
    }

    fn main_tbody(&self) -> Option<&html_parser::Node> {
        let Some(node) = self.main_table() else { return None };
        let Some(elm) = node.element() else { return None };
        elm.children.iter().find(|node| {
            let Some(elm) = node.element() else { return false };
            elm.name == TBODY
        })
    }

    pub async fn parse(&self) -> Result<Vec<Vec<table::Cell>>, ParsingError> {
        let Some(tbody) = self.main_tbody().map(|tb| tb.element()).flatten() else {
            return Err(ParsingError::NoTbody)
        };

        let mut x_jumping_conds: Vec<table::XJump> = vec![];
        let mut schema: Vec<Vec<table::Cell>> = vec![];
        let mut y = 0;

        let styles_text;
        let mut styles_input;
        let mut styles_parser;
        let mut styles = None;

        if let Some(styles_node) = self.style() {
            styles_text = parse::node::text::nested_as_string(styles_node, " ");
            styles_input = cssparser::ParserInput::new(&styles_text);
            styles_parser = cssparser::Parser::new(&mut styles_input);
            styles = Some(parse::css::Sheet::selectors(&mut styles_parser));
        }

        'row: for node_row in tbody.children.iter() {
            let Some(elm_row) = node_row.element() else { continue };

            let is_not_tr = elm_row.name != TR;
            if is_not_tr {
                continue;
            }

            let row_is_only_freezebar_cells = elm_row.children.iter()
                .filter_map(|td| td.element())
                .all(|td| td.classes.join(" ").contains(FREEZEBAR_CELL));
            if row_is_only_freezebar_cells {
                continue;
            };

            let style_string = elm_row.attributes
                .get(STYLE)
                .map(|opt| opt.clone())
                .flatten();
            
            if let Some(style_string) = style_string {
                let mut input = cssparser::ParserInput::new(&style_string);
                let mut parser = cssparser::Parser::new(&mut input);
                let props = parse::css::Properties::hashmap(&mut parser);
                if let Some(values) = props.get(HEIGHT) {
                    for value in values {
                        match value {
                            data::css::Value::Dimension(dim) => {
                                if dim.value < 3.0 { continue 'row; }
                            },
                            _ => ()
                        }
                    }
                }
            }

            let mut cells = vec![];
            let mut x = 0;

            for node_cell in elm_row.children.iter() {
                let Some(elm_cell) = node_cell.element() else { continue };

                let is_not_td = elm_cell.name != TD;
                if is_not_td {
                    continue;
                }

                let has_freezebar_class = elm_cell.classes
                    .join(" ")
                    .contains(FREEZEBAR_CELL);
                if has_freezebar_class {
                    continue;
                }

                loop {
                    let mut performed_jumps_count = 0;

                    for condition in x_jumping_conds.iter_mut() {
                        if {
                            !condition.is_done
                            && condition.at_x == x
                            && condition.at_y == y
                        } {
                            x += condition.by;
                            condition.done();
                            performed_jumps_count += 1;
                        }
                    };

                    if performed_jumps_count < 1 {
                        break;
                    }
                }

                let colspan_string = elm_cell.attributes
                    .get(COLSPAN)
                    .map(|opt| opt.clone())
                    .flatten()
                    .unwrap_or(ZERO.to_string());
                let colspan = colspan_string
                    .parse::<usize>()
                    .unwrap_or(0);
                let cell_width = {
                    if colspan < 1 { 1 }
                    else { colspan }
                };

                let rowspan_string = elm_cell.attributes
                    .get(ROWSPAN)
                    .map(|opt| opt.clone())
                    .flatten()
                    .unwrap_or(ZERO.to_string());
                let rowspan = rowspan_string
                    .parse::<usize>()
                    .unwrap_or(0);

                let text = parse::node::text::nested_as_string(node_cell, " ");

                let color = {
                    if let Some(styles) = styles.as_ref() {
                        parse::css::get_key_from_classes(
                            BACKGROUND_COLOR,
                            &elm_cell.classes,
                            styles
                        )
                            .map(|value| parse::css::get_any_hash_in_value(value))
                            .flatten()
                            .map(|color| format!("#{}", color))
                            .unwrap_or(DEFAULT_CELL_COLOR.to_string())
                    } else {
                        DEFAULT_CELL_COLOR.to_string()
                    }
                };

                let cell = table::Cell {
                    x,
                    y,
                    colspan,
                    rowspan,
                    text,
                    color
                };

                if cell.does_hit_next_rows() {
                    let mut future_y = y + 1;

                    for _ in 0..cell.row_hits() {
                        let jump = table::XJump {
                            at_x: x,
                            at_y: future_y,
                            by: cell_width,
                            is_done: false,
                        };
    
                        x_jumping_conds.push(jump);
                        future_y += 1;
                    }
                }

                cells.push(cell);

                x += cell_width;
            }

            schema.push(cells);

            y += 1;
        }

        Ok(schema)
    }
}