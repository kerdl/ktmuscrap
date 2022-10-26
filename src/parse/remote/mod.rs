use log::{info, debug};
use chrono::NaiveDate;
use html_parser::{Dom, Element, Node};
use actix_web::web::Bytes;
use tokio::sync::RwLock;
use std::{sync::Arc, path::{Path, PathBuf}, collections::HashMap};

use crate::{data::schedule::raw::{Zip, Type}, fs, DynResult};



pub struct Html {
    dom: Dom
}
impl Html {
    pub fn from_string(string: &str) -> DynResult<Html> {
        Ok(Html {
            dom: Dom::parse(string)?
        })
    }

    pub async fn from_path(path: &PathBuf) -> DynResult<Html> {
        let string = tokio::fs::read_to_string(path).await?;
        Html::from_string(&string)
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

            info!("tbody finder says: {}", node.element().unwrap().name);

            let is_tbody = node.element().unwrap().name == "tbody";

            is_tbody
        })
    }

    fn weekdays_row(&self) -> Option<&Node> {
        self.main_tbody()?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tr = node.element().unwrap().name == "tr";
            
            is_tr
        })
    }

    /// ## Get base date this schedule is for
    pub fn base_date(&self) -> Option<NaiveDate> {
        for node in self.main_tbody().unwrap() {
            if node.element().is_none() { continue; }

            let element = node.element().unwrap();
            
            info!("ELEMENT");

            info!("    NAME {:?}", element.name);
            info!("    ID {:?}", element.id);
            info!("    CLASSES {:?}", element.classes);
            info!("    ATTRS {:?}", element.attributes);
            info!("    CHILDREN {:?}", element.children);
        }

        todo!()
    }
}

pub struct HtmlContainer {
    pub list: Vec<Html>
}
impl HtmlContainer {
    pub fn new(list: Vec<Html>) -> HtmlContainer {
        HtmlContainer { list }
    }

    pub async fn from_paths(paths: &Vec<PathBuf>) -> DynResult<HtmlContainer> {
        let mut list = vec![];

        for path in paths {
            let html = Html::from_path(path).await?;
            list.push(html)
        }

        let container = HtmlContainer::new(list);

        Ok(container)
    }

    pub fn latest(&self) -> Option<(NaiveDate, &Html)> {
        let mut date_map: HashMap<NaiveDate, &Html> = HashMap::new();

        for html in self.list.iter() {
            let date = html.base_date();

            if date.is_none() {
                continue;
            }

            date_map.insert(date.unwrap(), html);
        }

        return date_map.into_iter()
            .max_by_key(|date_html: &(NaiveDate, &Html)| {
                let date = date_html.0;
                date
            })
    }
}

pub async fn parse(schedule: Arc<RwLock<Zip>>) -> DynResult<()> {
    let schedule = schedule.read().await;

    let html_container = schedule.to_html_container().await?;
    html_container.latest();

    Ok(())
}