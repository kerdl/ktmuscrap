use log::{info, debug};
use chrono::NaiveDate;
use html_parser::{Dom, Node};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf, collections::HashMap};

use crate::{
    data::{
        weekday::Weekday,
        schedule::{
            raw::Zip, 
            remote::table::NumTime
        }
    }, SyncResult
};
use super::{date, time, num};


#[derive(Debug, Clone)]
pub struct Html {
    dom: Dom,
    base_date: Option<NaiveDate>,
    time_table: Option<Vec<NumTime>>,

}
impl Html {
    pub fn new(
        dom: Dom,
        base_date: Option<NaiveDate>,
        time_table: Option<Vec<NumTime>>
    ) -> Html {

        Html { dom, base_date, time_table }
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

        Ok(Html::new(std::mem::take(&mut *dom_write_lock), None, None))
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

    /// # Get node text
    /// 
    /// < td ... >`text1` < br > `text2` < /td >
    /// - where `text1 text2` is what being returned
    fn get_node_text<'a>(&'a self, node: &'a Node) -> Option<String> {
        let mut texts = vec![];

        for node in node.element()?.children.iter() {
            if node.text().is_none() { continue; }
            
            texts.push(node.text().unwrap())
        }

        Some(texts.join(" "))
    }

    fn has_weekday(&self, node: &Node) -> bool {
        let mut is_found = false;

        for node in node.element().unwrap().children.iter() {

            let text = self.get_node_text(node);

            if text.is_none() {
                continue;
            }

            if text.as_ref().unwrap().is_empty() {
                continue;
            }

            let weekday = date::remove(&text.unwrap());

            if Weekday::guess(&weekday).is_some() {
                is_found = true;
                break;
            }
        }
        
        is_found
    } 

    fn weekdays_row(&self) -> Option<&Node> {
        self.main_tbody()?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tr = node.element().unwrap().name == "tr";
            
            is_tr && self.has_weekday(node)
        })
    }

    /// ## Get base date this schedule is for
    pub fn base_date(&mut self) -> Option<NaiveDate> {
        if self.base_date.is_some() {
            return self.base_date
        }

        for node in self.weekdays_row()? {

            let text = self.get_node_text(node);

            if text.is_none() {
                continue;
            }

            let date = date::parse_dmy(&text?);

            if date.is_some() {
                self.base_date = date;
                return self.base_date
            }
        }

        None
    }
    
    fn has_time(&self, node: &Node) -> bool {
        let mut is_found = false;

        if self.has_weekday(node) {
            return false;
        }

        for node in node.element().unwrap().children.iter() {

            let text = self.get_node_text(node);

            if text.is_none() {
                continue;
            }

            if text.as_ref().unwrap().is_empty() {
                continue;
            }

            let range = time::parse_range_hm(&text.unwrap());

            if range.is_some() {
                is_found = true;
                break;
            }
        }
        is_found
    }

    pub fn time_row(&self) -> Option<&Node> {
        self.main_tbody()?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tr = node.element().unwrap().name == "tr";

            is_tr && self.has_time(node)
        })
    }

    pub fn time_table(&mut self) -> Option<&Vec<NumTime>> {
        let mut table = vec![];

        if self.time_table.is_some() {
            return Some(self.time_table.as_ref().unwrap())
        }

        for (index, node) in self.time_row()?
                            .element()?
                            .children.iter()
                            .enumerate() 
        {
            // "1 пара 8:30-9:55"
            let text = self.get_node_text(node);

            if text.is_none() {
                continue;
            }


            //  "1 пара"
            let named_num = time::remove(text.as_ref().unwrap());
            //  1
            let num = num::parse(&named_num);

            if num.is_none() { continue; }


            //  Option<NaiveTime(8, 30)..NaiveTime(9, 55)>
            let time = time::parse_range_hm(text.as_ref().unwrap());

            if time.is_none() { continue; }

            todo!();

            //let num_time_idx = NumTime::new(num?, time?, index);
            //table.push(num_time_idx);
        }

        self.time_table = Some(table);

        Some(self.time_table.as_ref().unwrap())
    }

    fn group_rows(&self) -> Option<Vec<&Node>> {
        todo!()
    }
}


pub struct HtmlContainer {
    pub list: Vec<Html>,
    pub old: Vec<Html>
}
impl HtmlContainer {
    pub fn new(list: Vec<Html>, old: Vec<Html>) -> HtmlContainer {
        HtmlContainer { list, old }
    }

    pub async fn from_paths(paths: Vec<PathBuf>) -> SyncResult<HtmlContainer> {
        let mut this = HtmlContainer::default();

        this.add_from_paths(paths).await?;

        Ok(this)
    }

    pub fn add_html(&mut self, html: Html) {
        self.list.push(html);
    }

    pub async fn add_from_path(
        &mut self, 
        path: PathBuf
    ) -> SyncResult<()> {

        let html = Html::from_path(&path).await?;
        self.add_html(html);

        Ok(())
    }

    pub async fn add_from_paths(
        &mut self, 
        paths: Vec<PathBuf>
    ) -> SyncResult<()> {

        let htmls = Arc::new(RwLock::new(vec![]));
        let htmls_ref = htmls.clone();

        let mut handles = vec![];

        for path in paths {
            let htmls_ref = htmls_ref.clone();

            let handle = tokio::spawn(async move {
                let htmls_ref = htmls_ref.clone();
                let path = path.clone();

                let html = Html::from_path(&path).await.unwrap();

                let mut htmls = htmls_ref.write().await;
                htmls.push(html);
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        for handle in handles {
            handle.await?;
        }

        // get writing lock for htmls list
        let mut htmls_write = htmls.write().await;
        // take everything from that list, move it
        // to this one
        let htmls = std::mem::take(&mut *htmls_write);

        // add everything to `self` container
        self.list.extend_from_slice(&htmls[..]);

        Ok(())
    }

    pub async fn latest(&mut self) -> Option<(NaiveDate, &mut Html)> {
        let mut date_map: HashMap<NaiveDate, &mut Html> = HashMap::new();

        for html in self.list.iter_mut() {
            let date = html.base_date();

            if date.is_none() {
                continue;
            }

            date_map.insert(date.unwrap(), html);
        }

        return date_map.into_iter()
            .max_by_key(|date_html: &(NaiveDate, &mut Html)| {
                let date = date_html.0;
                date
            })
    }
}
impl Default for HtmlContainer {
    fn default() -> Self {
        HtmlContainer::new(vec![], vec![])
    }
}

pub async fn parse(schedule: Arc<RwLock<Zip>>) -> SyncResult<()> {
    let schedule = schedule.read().await;

    let mut html_container = schedule.to_html_container().await?;

    let mut latest = html_container.latest().await;
    info!("latest: {}", latest.as_ref().unwrap().0);

    let time_row = latest.as_mut().unwrap().1.time_table();
    info!("time: {:?}", time_row.unwrap());

    Ok(())
}