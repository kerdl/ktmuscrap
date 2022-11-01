use log::info;
use html_parser::{Dom, Node, Error};
use htmlescape;
use std::path::PathBuf;

use crate::SyncResult;
use super::table::Parser as TableParser;


pub struct Parser {
    dom: Dom,
    table: Option<TableParser>,
}
impl Parser {
    pub fn new(
        dom: Dom,
        table: Option<TableParser>,
    ) -> Parser {

        Parser { dom, table }
    }

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

    pub fn table(&mut self) -> Option<&TableParser> {

        if self.table.is_some() {
            return Some(self.table.as_ref().unwrap())
        }

        for node in self.main_body()?.element()?.children.iter() {
            info!("{:?}", node)
        }

        todo!()
    }
}