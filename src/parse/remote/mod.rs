use log::{info, debug};
use chrono::NaiveDate;
use html_parser::{Dom, Node};
use tokio::sync::RwLock;
use std::{sync::Arc, path::PathBuf, collections::HashMap};

use crate::{data::schedule::{raw::Zip, weekday::Weekday}, DynResult, SyncResult};



#[derive(Debug, Clone)]
pub struct Html {
    dom: Dom

}
impl Html {
    pub fn new(dom: Dom) -> Html {
        Html { dom }
    }

    pub async fn from_string(string: String) -> DynResult<Html> {
        let dom = Arc::new(RwLock::new(Dom::default()));

        let dom_ref = dom.clone();
        let handle = tokio::task::spawn_blocking(move || {
            let parsed_dom = Dom::parse(&string).unwrap();

            let mut dom = dom_ref.blocking_write();
            *dom = parsed_dom
        });

        handle.await?;

        let mut dom_write_lock = dom.write().await;

        Ok(Html::new(std::mem::take(&mut *dom_write_lock)))
    }

    pub async fn from_path(path: &PathBuf) -> DynResult<Html> {
        let string = tokio::fs::read_to_string(path).await?;
        Html::from_string(string).await
    }

    async fn main_div(&self) -> Option<&Node> {
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

    async fn main_table(&self) -> Option<&Node> {
        self.main_div().await?.element()?.children.iter().find(|node| {
            if node.element().is_none() {
                return false
            }

            let is_table = node.element().unwrap().name == "table";

            is_table
        })
    }

    async fn main_tbody(&self) -> Option<&Node> {
        self.main_table().await?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tbody = node.element().unwrap().name == "tbody";

            is_tbody
        })
    }

    /// # Get node text
    /// 
    /// < td ... >`text`< /td >
    /// - where `text` is what being returned
    fn get_node_text<'a>(&'a self, node: &'a Node) -> Option<&str> {
        node.element()?.children.get(0)?.text()
    }

    async fn weekdays_row(&self) -> Option<&Node> {
        self.main_tbody().await?.element()?.children.iter().find(|node| {

            if node.element().is_none() {
                return false
            }

            let is_tr = node.element().unwrap().name == "tr";

            if !is_tr {
                return false;
            }

            let has_weekday = {
                let mut is_found = false;

                for node in node.element().unwrap().children.iter() {

                    if self.get_node_text(node).is_none() {
                        continue;
                    }

                    let text = self.get_node_text(node).unwrap();

                    let weekday = super::date::remove(text);

                    if Weekday::guess(&weekday).is_some() {
                        is_found = true;
                        break;
                    }
                }
                
                is_found
            };
            
            has_weekday
        })
    }

    /// ## Get base date this schedule is for
    pub async fn base_date(&self) -> Option<NaiveDate> {
        for node in self.weekdays_row().await? {

            let text = self.get_node_text(node);

            if text.is_none() {
                continue;
            }

            let date = super::date::parse_dmy(text?);

            if date.is_some() {
                return date
            }
        }

        None
    }
}


pub struct HtmlContainer {
    pub list: Arc<RwLock<Vec<Arc<Html>>>>
}
impl HtmlContainer {
    pub fn new(list: Arc<RwLock<Vec<Arc<Html>>>>) -> HtmlContainer {
        HtmlContainer { list }
    }

    pub async fn from_paths(paths: Vec<PathBuf>) -> SyncResult<Arc<HtmlContainer>> {
        let this = Arc::new(HtmlContainer::default());

        this.clone().add_from_paths(paths).await?;

        Ok(this)
    }

    pub async fn add_html(self: Arc<Self>, html: Html) {
        let mut list = self.list.write().await;
        list.push(Arc::new(html));
    }

    pub async fn add_from_path(
        self: Arc<Self>, 
        path: PathBuf
    ) -> DynResult<()> {

        let html = Html::from_path(&path).await?;
        self.add_html(html).await;

        Ok(())
    }

    pub async fn add_from_paths(
        self: Arc<Self>, 
        paths: Vec<PathBuf>
    ) -> SyncResult<()> {

        let mut handles = vec![];

        for path in paths {
            let self_ref = self.clone();

            let handle = tokio::spawn(async move {
                self_ref.add_from_path(path).await.unwrap();
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        for handle in handles {
            handle.await?;
        }

        Ok(())
    }

    pub async fn latest(self: Arc<Self>) -> Option<(NaiveDate, Arc<Html>)> {
        let mut date_map: HashMap<NaiveDate, Arc<Html>> = HashMap::new();

        for html in self.list.clone().read().await.iter() {
            let date = html.base_date().await;

            if date.is_none() {
                continue;
            }

            date_map.insert(date.unwrap(), html.clone());
        }

        return date_map.into_iter()
            .max_by_key(|date_html: &(NaiveDate, Arc<Html>)| {
                let date = date_html.0;
                date
            })
    }
}
impl Default for HtmlContainer {
    fn default() -> Self {
        HtmlContainer::new(Arc::new(RwLock::new(vec![])))
    }
}

pub async fn parse(schedule: Arc<RwLock<Zip>>) -> SyncResult<()> {
    let schedule = schedule.read().await;

    let html_container = schedule.to_html_container().await?;
    let latest = html_container.latest().await;

    info!("latest: {}", latest.unwrap().0);

    Ok(())
}