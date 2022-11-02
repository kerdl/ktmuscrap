use log::info;
use derive_new::new;
use html_parser::{Dom, Node, Error};
use htmlescape;
use std::path::PathBuf;

use crate::{REGEX, SyncResult, data::schedule::fulltime::html::HeaderTable};
use super::{tables::Parser as TablesParser, super::node};


enum Lookup {
    Header,
    Table
}

#[derive(new)]
pub struct Parser {
    dom: Dom,
    tables: Option<TablesParser>,
}
impl Parser {
    pub async fn from_string(html_text: String, ) -> SyncResult<Parser> {
        let handle = tokio::task::spawn_blocking(move || -> Result<Dom, Error> {
            Dom::parse(&html_text)
        });

        let dom = handle.await??;
        let table = None;

        let parser = Parser::new(dom, table);

        Ok(parser)
    }

    pub async fn from_path(path: PathBuf) -> SyncResult<Parser> {
        let html_text = tokio::fs::read_to_string(path).await?;
        let decoded_html_text = htmlescape::decode_html(&html_text).unwrap();
        Parser::from_string(decoded_html_text).await
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

    pub fn tables(&mut self) -> Option<&mut TablesParser> {

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

        self.tables = Some(TablesParser::from_header_tables(header_tables));

        Some(self.tables.as_mut().unwrap())
    }
}