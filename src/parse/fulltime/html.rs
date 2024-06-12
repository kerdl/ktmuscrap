use derive_new::new;
use html_parser::{Dom, Node, Error};
use htmlescape;
use chrono::NaiveTime;
use std::path::PathBuf;
use std::{collections::HashMap, ops::Range};

use crate::data::Weekday;
use crate::{
    REGEX,
    data::schedule::raw::{
        self,
        fulltime::html::{HeaderTable, HeaderSpanTable},
        table
    }
};
use super::{
    tables,
    super::node
};


enum Lookup {
    Header,
    Table
}

#[derive(Debug)]
pub enum LoadingError {
    Io(tokio::io::Error),
    Parsing(Error)
}
impl From<tokio::io::Error> for LoadingError {
    fn from(err: tokio::io::Error) -> Self {
        Self::Io(err)
    }
}
impl From<Error> for LoadingError {
    fn from(err: Error) -> Self {
        Self::Parsing(err)
    }
}

#[derive(new, Debug, Clone)]
pub struct Parser {
    sc_type: raw::Type,
    dom: Dom,
    tables: Option<tables::Parser>,
}
impl Parser {
    pub async fn from_string(html_text: String, sc_type: raw::Type) -> Result<Parser, Error> {
        let handle = tokio::task::spawn_blocking(move || -> Result<Dom, Error> {
            // SLOW AS FUCK
            // ~1s ON 480 KB HTML
            Dom::parse(&html_text)
        });

        let dom = handle.await.unwrap()?;
        let table = None;

        let parser = Parser::new(sc_type, dom, table);

        Ok(parser)
    }

    pub async fn from_path(path: PathBuf, sc_type: raw::Type) -> Result<Parser, LoadingError> {
        let html_text = tokio::fs::read_to_string(path).await?;
        let decoded_html_text = htmlescape::decode_html(&html_text).unwrap();
        let parser = Parser::from_string(decoded_html_text, sc_type).await?;

        Ok(parser)
    }

    fn main_html(&self) -> Option<&Node> {
        self.dom.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "html"
        })
    }

    fn main_body(&self) -> Option<&Node> {
        self.main_html()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "body"
        })
    }

    fn html_table_to_vec(&self, node: &Node) -> Option<Vec<Vec<String>>> {
        let mut rows: Vec<Vec<String>> = vec![];

        for row in node.element()?.children.iter() {
            let mut cells: Vec<String> = vec![];

            for cell in row.element().unwrap().children.iter() {
                cells.push(node::text::nested_as_string(cell, " "))
            }

            rows.append(&mut vec![cells]);
        }

        Some(rows)
    }

    pub fn tables(&mut self) -> Option<&mut tables::Parser> {
        if self.tables.is_some() {
            return Some(self.tables.as_mut().unwrap())
        }

        let mut lookup = Lookup::Header;
        let mut header: Option<String> = None;
        let mut header_tables: Vec<HeaderTable> = vec![];

        for node in self.main_body()?.element()?.children.iter() {
            if node.element().is_none() {
                continue;
            }

            match lookup {
                Lookup::Header => {
                    if node.element().unwrap().name != "p" {
                        continue;
                    }

                    let text = node::text::nested_as_string(node, " ");

                    if !REGEX.group.is_match(&text) {
                        continue;
                    }

                    header = Some(text);

                    lookup = Lookup::Table;
                    continue;
                }
                Lookup::Table => {
                    if node.element().unwrap().name != "table" {
                        continue;
                    }

                    let table = self.html_table_to_vec(node);

                    let header_table = HeaderTable::new(
                        header.take().unwrap(), 
                        table.unwrap()
                    );
                    header_tables.push(header_table);

                    lookup = Lookup::Header;
                    continue;
                }
            }
        }

        self.tables = Some(
            tables::Parser::from_header_tables(
                header_tables,
                self.sc_type.clone(),
            )
        );

        Some(self.tables.as_mut().unwrap())
    }
}

#[derive(new, Debug, Clone)]
pub struct TchrDailyParser {
    dom: Dom,
    tables: Option<tables::TchrDailyParser>,
}
impl TchrDailyParser {
    pub async fn from_string(html_text: String,) -> Result<Self, Error> {
        let handle = tokio::task::spawn_blocking(move || -> Result<Dom, Error> {
            // SLOW AS FUCK
            // ~1s ON 480 KB HTML
            Dom::parse(&html_text)
        });

        let dom = handle.await.unwrap()?;
        let table = None;

        let parser = Self::new(dom, table);

        Ok(parser)
    }

    pub async fn from_path(path: PathBuf) -> Result<Self, LoadingError> {
        let html_text = tokio::fs::read_to_string(path).await?;
        let decoded_html_text = htmlescape::decode_html(&html_text).unwrap();
        let parser = Self::from_string(decoded_html_text).await?;

        Ok(parser)
    }

    fn main_html(&self) -> Option<&Node> {
        self.dom.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "html"
        })
    }

    fn main_body(&self) -> Option<&Node> {
        self.main_html()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "body"
        })
    }

    fn main_p_header(&self) -> Option<&Node> {
        self.main_body()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "p"
        })
    }

    fn main_span_header(&self) -> Option<&Node> {
        self.main_p_header()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "span"
        })
    }

    fn main_table(&self) -> Option<&Node> {
        let main_body = self.main_body();
        if main_body.is_none() { return None; }
        main_body.unwrap().element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "table"
        })
    }

    fn html_table_to_vec(&self, node: &Node) -> Option<Vec<Vec<table::Cell>>> {
        let mut y = 0;
        let mut rows: Vec<Vec<table::Cell>> = vec![];

        // info to indicate X axis jump:
        //   - on what X
        //   - on what Y
        //   - by how much
        //
        // used for cells that are taking
        // more than 1 row
        let mut x_jumping_conds: Vec<table::XJump> = vec![];

        for row in node.element()?.children.iter() {
            let mut x = 0;
            let mut cells: Vec<table::Cell> = vec![];

            for cell in row.element().unwrap().children.iter() {
                // look for conditions that are
                // sometimes put a the end
                // of this "for cell" loop
                loop {
                    let mut performed_jumps_count = 0;

                    for condition in x_jumping_conds.iter_mut() {
                        if {
                            // if isn't performed
                            // previously
                            !condition.is_done
                            // and if current X axis
                            // is exactly the same 
                            // as in condition
                            && condition.at_x == x
                            // and if current Y axis
                            // is exactly the same 
                            // as in condition
                            && condition.at_y == y
                        } {
                            // increment X axis (jump)
                            // by the value inside
                            // condition
                            x += condition.by;
                            // mark this condition as done
                            condition.done();
                            performed_jumps_count += 1;
                        }
                    };

                    if performed_jumps_count < 1 {
                        break;
                    }
                }

                let zero = "0".to_string();
                let some_zero = Some(zero.clone());

                let colspan = {
                    cell.element()?
                    .attributes.get("colspan")
                    .unwrap_or(&some_zero).as_ref()
                    .unwrap_or(&zero)
                };
                let colspan = colspan.parse::<usize>().unwrap_or(0);
                let rowspan = {
                    cell.element()?
                    .attributes.get("rowspan")
                    .unwrap_or(&some_zero).as_ref()
                    .unwrap_or(&zero)
                };
                let rowspan = rowspan.parse::<usize>().unwrap_or(0);
                let cell_width = {
                    if colspan < 1 { 1 }
                    else { colspan }
                };
                let cell_height = {
                    if rowspan < 1 { 1 }
                    else { rowspan }
                };

                let text = node::text::nested_as_string(cell, " ");

                let parsed_cell = table::Cell::new(x, y, colspan, rowspan, text);

                if parsed_cell.hits_next_rows() {
                    // this cell definitely hits
                    // the next row
                    let mut future_y = y + 1;

                    // for each affected row
                    for _ in 0..parsed_cell.hits() {
                        // create X axis jump condition,
                        // that will execute later
                        // in "for cell" loop
                        let jump = table::XJump {
                            at_x:    x,
                            at_y:    future_y,
                            by:      cell_width,
                            is_done: false,
                        };

                        x_jumping_conds.push(jump);

                        future_y += 1;
                    }
                }

                cells.push(parsed_cell);
                
                x += cell_width;
            }

            rows.append(&mut vec![cells]);

            y += 1;
        }

        Some(rows)
    }

    pub fn tables(&mut self) -> Option<&mut tables::TchrDailyParser> {
        if self.tables.is_some() {
            return Some(self.tables.as_mut().unwrap())
        }

        let header = node::text::nested_as_string(self.main_span_header()?, " ");
        let table = {
            let main_table = self.main_table();
            if main_table.is_none() {
                None
            } else {
                self.html_table_to_vec(main_table.unwrap())
            }
        }?;
        let header_tables = HeaderSpanTable::new(header, table);

        self.tables = Some(
            tables::TchrDailyParser::from_header_tables(
                header_tables
            )
        );

        Some(self.tables.as_mut().unwrap())
    }
}

#[derive(new, Debug, Clone)]
pub struct TchrWeeklyParser {
    dom: Dom,
    tables: Option<tables::TchrWeeklyParser>,
}
impl TchrWeeklyParser {
    pub async fn from_string(html_text: String) -> Result<Self, Error> {
        let handle = tokio::task::spawn_blocking(move || -> Result<Dom, Error> {
            // SLOW AS FUCK
            // ~1s ON 480 KB HTML
            Dom::parse(&html_text)
        });

        let dom = handle.await.unwrap()?;
        let table = None;

        let parser = Self::new(dom, table);

        Ok(parser)
    }

    pub async fn from_path(path: PathBuf) -> Result<Self, LoadingError> {
        let html_text = tokio::fs::read_to_string(path).await?;
        let decoded_html_text = htmlescape::decode_html(&html_text).unwrap();
        let parser = Self::from_string(decoded_html_text).await?;

        Ok(parser)
    }

    fn main_html(&self) -> Option<&Node> {
        self.dom.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "html"
        })
    }

    fn main_body(&self) -> Option<&Node> {
        self.main_html()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            node.element().unwrap().name == "body"
        })
    }

    fn html_table_to_vec(&self, node: &Node) -> Option<Vec<Vec<String>>> {
        let mut rows: Vec<Vec<String>> = vec![];

        for row in node.element()?.children.iter() {
            let mut cells: Vec<String> = vec![];

            for cell in row.element().unwrap().children.iter() {
                cells.push(node::text::nested_as_string(cell, " "))
            }

            rows.append(&mut vec![cells]);
        }

        Some(rows)
    }

    pub fn tables(&mut self) -> Option<&mut tables::TchrWeeklyParser> {
        if self.tables.is_some() {
            return Some(self.tables.as_mut().unwrap())
        }

        let mut lookup = Lookup::Header;
        let mut header: Option<String> = None;
        let mut header_tables: Vec<HeaderTable> = vec![];

        for node in self.main_body()?.element()?.children.iter() {
            if node.element().is_none() {
                continue;
            }

            match lookup {
                Lookup::Header => {
                    if node.element().unwrap().name != "p" {
                        continue;
                    }

                    let text = node::text::nested_as_string(node, " ");

                    if !REGEX.teacher.is_match(&text) {
                        continue;
                    }

                    header = Some(text);

                    lookup = Lookup::Table;
                    continue;
                }
                Lookup::Table => {
                    if node.element().unwrap().name != "table" {
                        continue;
                    }

                    let table = self.html_table_to_vec(node);

                    let header_table = HeaderTable::new(
                        header.take().unwrap(), 
                        table.unwrap()
                    );
                    header_tables.push(header_table);

                    lookup = Lookup::Header;
                    continue;
                }
            }
        }

        self.tables = Some(
            tables::TchrWeeklyParser::from_header_tables(
                header_tables
            )
        );

        Some(self.tables.as_mut().unwrap())
    }
}