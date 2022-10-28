use html_parser::{Dom, Node};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf};

use crate::{
    data::schedule::raw::table, 
    SyncResult
};
use super::node;


/// # 1st step of parsing remote schedule
/// 
/// ## In a nutshell
/// - we load html with `from_path()` or `from_string()`
/// ```
///     Html::from_path("./schedule.html");
/// ```
/// 
/// - these functions also parse it with
/// `html_parser::Dom` in separate thread
/// 
/// - `main_div()` searches for specific div
/// ```notrust
///     <body>
///         <div>    ← ✓
///             ...  ← ✓
///         </div>   ← ✓
///     </body>
/// ```
/// 
/// - `main_table()` searches for specific 
/// table in div
/// ```notrust
///     <div>
///         <table>   ← ✓
///             ...   ← ✓
///         </table>  ← ✓
///     </div>
/// ```
/// 
/// - `main_tbody()` searches for the final 
/// table body in table
/// ```notrust
///     <table>
///         <thead>
///             ...
///         </thead> 
///         <tbody>   ← ✓
///             ...   ← ✓
///         </tbody>  ← ✓
///     </table>
/// ```
/// 
/// - `table()` converts it to 
/// `crate::data::schedule::raw::table::Body`
/// type, that is easier to process
#[derive(Debug, Clone)]
pub struct Html {
    dom: Dom,
    table: Option<table::Body>,
}
impl Html {
    pub fn new(
        dom: Dom,
        table: Option<table::Body>,
    ) -> Html {

        Html { dom, table }
    }

    pub async fn from_string(string: String) -> SyncResult<Html> {
        let dom = Arc::new(RwLock::new(Dom::default()));

        let dom_ref = dom.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let parsed_dom = Dom::parse(&string).unwrap();

            let mut dom = dom_ref.blocking_write();
            *dom = parsed_dom
        });

        handle.await?;

        let mut dom_write_lock = dom.write().await;

        let dom = std::mem::take(&mut *dom_write_lock);
        let table = None;

        Ok(Html::new(dom, table))
    }

    pub async fn from_path(path: &PathBuf) -> SyncResult<Html> {
        let string = tokio::fs::read_to_string(path).await?;
        Html::from_string(string).await
    }

    fn main_div(&self) -> Option<&Node> {
        self.dom.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }
            
            let is_div = node
                .element()
                .unwrap()
                .name == "div";
            let is_grid_container = node
                .element()
                .unwrap()
                .classes
                .contains(&"grid-container".to_owned());
            
            is_div && is_grid_container
        })
    }

    fn main_table(&self) -> Option<&Node> {
        self.main_div()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            let is_table = node.element().unwrap().name == "table";

            is_table
        })
    }

    fn main_tbody(&self) -> Option<&Node> {
        self.main_table()?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tbody = node.element().unwrap().name == "tbody";

            is_tbody
        })
    }

    pub fn table(&mut self) -> Option<&table::Body> {

        if self.table.is_some() {
            return Some(self.table.as_ref().unwrap())
        }

        let mut schema: Vec<Vec<table::Cell>> = vec![];

        for row in self.main_tbody()?.element()?.children.iter() {
            
            let mut cells: Vec<table::Cell> = vec![];


            if row.element().is_none() { continue; }
            if row.element()?.name != "tr" { continue; }


            for cell in row.element()?.children.iter() {

                if cell.element().is_none() { continue; }
                if cell.element()?.name != "td" { continue; }
                if cell.element()?.classes.contains(
                    &"freezebar-cell".to_owned()
                ) { continue; }


                let zero = "0".to_string();
                let some_zero = Some(zero.clone());


                let colspan = {
                    cell.element()?
                    .attributes.get("colspan")
                    .unwrap_or(&some_zero).as_ref()
                    .unwrap_or(&zero)
                };
                let colspan = colspan.parse::<u32>().unwrap_or(0);

                let rowspan = {
                    cell.element()?
                    .attributes.get("rowspan")
                    .unwrap_or(&some_zero).as_ref()
                    .unwrap_or(&zero)
                };
                let rowspan = rowspan.parse::<u32>().unwrap_or(0);

                let text = node::text::nested_as_string(cell, " ");

                let cell_i_would_like_to_fuck = table::Cell::new(
                    colspan, 
                    rowspan, 
                    text
                );

                cells.push(cell_i_would_like_to_fuck);
            }

            if !cells.is_empty() {
                schema.push(cells);
            }
        }

        let body = table::Body::new(schema);

        self.table = Some(body);

        Some(self.table.as_ref().unwrap())
    }
}
