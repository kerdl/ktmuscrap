use html_parser::{Dom, Node};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf};

use crate::{
    data::schedule::raw::table, 
    SyncResult
};
use super::{
    node, 
    table::Parser as TableParser
};


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
/// type, that is easier to process, and
/// stores this converted table body
/// in `self.table`, a reference to which
/// it will return later on to not recalculate
/// the same thing
#[derive(Debug, Clone)]
pub struct Parser {
    dom: Dom,
    pub path: PathBuf,
    table: Option<TableParser>,
}
impl Parser {
    pub fn new(
        dom: Dom,
        path: PathBuf,
        table: Option<TableParser>,
    ) -> Parser {

        Parser { dom, path, table }
    }

    /// # Load from HTML text
    /// 
    /// ## Params
    /// - `string`: HTML text itself
    /// - `path`: location of this file
    /// in file system
    /// 
    /// ## Why `Result`
    /// - spawned thread, 
    /// which runs `Dom` parsing, 
    /// may fail
    pub async fn from_string(string: String, path: PathBuf) -> SyncResult<Parser> {

        // spawned thread will send converted data here
        let dom = Arc::new(RwLock::new(Dom::default()));

        // make a reference to it,
        // we'll pass this ref to separate thread
        let dom_ref = dom.clone();

        // `spawn_blocking` spawns the task 
        // in a separate thread,
        // but only allows synchronous code
        let handle = tokio::task::spawn_blocking(move || -> SyncResult<()> {

            // actually parse it
            let parsed_dom = Dom::parse(&string)?;

            // acquire lock on this `dom` variable
            let mut dom = dom_ref.blocking_write();
            // send data there
            *dom = parsed_dom;

            Ok(())
        }); // `dom` lock releases

        // wait until the spawned task finishes
        handle.await??;

        // acquire lock on dom variable
        let mut dom_write_lock = dom.write().await;

        // get rid of all this Arc<RwLock> bullshit,
        // leaving us with clean `Dom`
        let dom = std::mem::take(&mut *dom_write_lock);
        let table = None;

        Ok(Parser::new(dom, path, table))
    }

    /// # Load from HTML file
    pub async fn from_path(path: PathBuf) -> SyncResult<Parser> {
        let string = tokio::fs::read_to_string(&path).await?;

        Parser::from_string(string, path).await
    }

    /// # Search for `div` with main content
    fn main_div(&self) -> Option<&Node> {
        self.dom.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }
            
            let is_div = {
                node
                .element()
                .unwrap()
                .name == "div"
            };
            let is_grid_container = {
                node
                .element()
                .unwrap()
                .classes
                .contains(&"grid-container".to_owned())
            };
            
            is_div && is_grid_container
        })
    }

    /// # Search for `table` with main content in `div`
    fn main_table(&self) -> Option<&Node> {
        self.main_div()?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            let is_table = node.element().unwrap().name == "table";

            is_table
        })
    }

    /// # Search for `tbody` with main content in `table`
    fn main_tbody(&self) -> Option<&Node> {
        self.main_table()?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tbody = node.element().unwrap().name == "tbody";

            is_tbody
        })
    }

    /// # Convert dom to simpler `table::Body`
    /// 
    /// - we don't need all the complex data
    /// HTML holds inside itseld 
    ///     - nested texts, 
    ///     - styled separators, 
    ///     - etc.
    /// 
    /// - so, instead of working with raw HTML,
    /// we take only useful data for easier parsing
    ///     - how wide or tall the cells are: `colspan` and `rowspan`,
    ///     - the nested text inside cells, joined in one string
    pub fn table(&mut self) -> Option<&mut TableParser> {

        // if the conversion had already been made
        if self.table.is_some() {
            // return reference to converted table
            return Some(self.table.as_mut().unwrap())
        }

        // 2d array, represents a table
        let mut schema: Vec<Vec<table::Cell>> = vec![];

        // info to indicate X axis jump:
        //   - on what X
        //   - on what Y
        //   - by how much
        //
        // used for cells that are taking
        // more than 1 row
        let mut x_jumping_conds: Vec<table::XJump> = vec![];

        // ↕ Y axis of a table cell
        let mut y = 0;

        // iterate rows
        //
        // > ■■■■■■■■■■■ ↓
        //   □ □ □ □ □ □ ↓
        //   □ □ □ □ □ □ ↓
        for row in self.main_tbody()?.element()?.children.iter() {

            // skip if no element
            if row.element().is_none() { continue; }
            // skip if tag is not <tr>
            if row.element()?.name != "tr" { continue; }
            // skip if entire row is made out
            // of "freezebar-cell" cells
            if row.element()?.children.iter().all(|cell| 
                cell.element().is_some() 
                && cell.element().unwrap().classes.contains(
                    &"freezebar-cell".to_owned()
                )
            ) { continue; }


            // cells of this row (columns)
            let mut cells: Vec<table::Cell> = vec![];

            // ↔ X axis of a table cell
            let mut x = 0;

            // iterate cells (columns)
            // ⌄
            // ■ □ □ □ □ □
            // → → → → → →
            for cell in row.element()?.children.iter() {

                // skip if no element
                if cell.element().is_none() { continue; }
                // skip if tag is not <td>
                if cell.element()?.name != "td" { continue; }
                // skip if this <td> has "freezebar-cell" class
                // (it's actually just a separator)
                if cell.element()?.classes.contains(
                    &"freezebar-cell".to_owned()
                ) { continue; }


                // look for conditions that are
                // sometimes put a the end
                // of this "for cell" loop
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
                    }
                };

                let zero = "0".to_string();
                let some_zero = Some(zero.clone());


                // get colspan value in <td> attributes
                // (how wide this cell is)
                //
                // colspan = 0 or 1
                // ■ □ □ □ □
                // □ □ □ □ □
                // □ □ □ □ □
                // □ □ □ □ □
                // □ □ □ □ □
                //
                // colspan = 3
                // ■■■■■ □ □
                // □ □ □ □ □
                // □ □ □ □ □
                // □ □ □ □ □
                // □ □ □ □ □
                let colspan = {
                    cell.element()?
                    .attributes.get("colspan")
                    .unwrap_or(&some_zero).as_ref()
                    .unwrap_or(&zero)
                };
                let colspan = colspan.parse::<usize>().unwrap_or(0);
                let cell_width = {
                    if colspan < 1 { 1 }
                    else { colspan }
                };

                // get rowspan value in <td> attributes
                // (how tall this cell is)
                //
                // rowspan = 0 or 1
                // ■ □ □ □ □ □
                // □ □ □ □ □ □
                // □ □ □ □ □ □
                // □ □ □ □ □ □
                // □ □ □ □ □ □
                //
                // rowspan = 3 
                // █ □ □ □ □ □
                // █ □ □ □ □ □
                // █ □ □ □ □ □
                // □ □ □ □ □ □
                // □ □ □ □ □ □
                let rowspan = {
                    cell.element()?
                    .attributes.get("rowspan")
                    .unwrap_or(&some_zero).as_ref()
                    .unwrap_or(&zero)
                };
                let rowspan = rowspan.parse::<usize>().unwrap_or(0);
                let cell_height = {
                    if rowspan < 1 { 1 }
                    else { rowspan }
                };


                // we need colspan and rowspan to tell
                //
                //  - which days of week (colspan)
                //  are affected by ONE MERGED CELL
                //
                //      a merged cell affects 1-kDD-10 only
                //      on monday and tuesday
                //                      ↓
                //               Monday  Tuesday   Wednesday
                //      1-kDD-10 ███████████████    □□□□□□
                //      1-kDD-12 □□□□□□   □□□□□□    □□□□□□
                //      1-kDD-14 □□□□□□   □□□□□□    □□□□□□
                //
                //
                //  - which groups (rowspan) are
                //  affected by ONE MERGED CELL
                //
                //               Monday  Tuesday   Wednesday
                //      1-kDD-10 ██████   □□□□□□    □□□□□□
                //      1-kDD-12 ██████   □□□□□□    □□□□□□
                //      1-kDD-14 ██████   □□□□□□    □□□□□□
                //                 ↑
                //      a merged cell affects 1-kDD-(10, 12, 14)
                //      on monday only

                let text = node::text::nested_as_string(cell, " ");

                // construct clean cell only with data we need
                let clean_cell = table::Cell::new(
                    x,
                    y,
                    colspan, 
                    rowspan, 
                    text
                );

                if clean_cell.hits_next_rows() {
                    // this cell definitely hits
                    // the next row
                    let mut future_y = y + 1;

                    // for each affected row
                    for _ in 0..clean_cell.hits() {

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

                cells.push(clean_cell);

                // we're going to the next cell
                x += cell_width;
            }

            // if not all cells in this row 
            // were filtered out
            if !cells.is_empty() {
                // push this row to the full table schema
                schema.push(cells);
            }

            // we're going to the next row
            y += 1;
        }

        let tbody = table::Body::new(schema);
        let parser = TableParser::from_table(tbody);

        self.table = Some(parser);

        Some(self.table.as_mut().unwrap())
    }
}
