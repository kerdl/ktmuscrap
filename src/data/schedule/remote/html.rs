use chrono::NaiveDate;
use tokio::sync::RwLock;
use std::{path::PathBuf, sync::Arc, collections::HashMap};

use crate::{parse::remote::html, SyncResult};


pub struct Container {
    pub list: Vec<html::Parser>,
    pub old: Vec<html::Parser>
}
impl Container {
    pub fn new(list: Vec<html::Parser>, old: Vec<html::Parser>) -> Container {
        Container { list, old }
    }

    pub async fn from_paths(paths: Vec<PathBuf>) -> SyncResult<Container> {
        let mut this = Container::default();

        this.add_from_paths(paths).await?;

        Ok(this)
    }

    pub fn add_html(&mut self, html: html::Parser) {
        self.list.push(html);
    }

    pub async fn add_from_path(
        &mut self, 
        path: PathBuf
    ) -> SyncResult<()> {

        let html = html::Parser::from_path(&path).await?;
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

                let html = html::Parser::from_path(&path).await.unwrap();

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
        self.list.extend_from_slice(&htmls);

        Ok(())
    }

    pub async fn latest(&mut self) -> Option<(NaiveDate, &mut html::Parser)> {

        let mut date_map: HashMap<NaiveDate, &mut html::Parser> = HashMap::new();

        for parser in self.list.iter_mut() {

            let mut table_parser = parser.to_table_parser();

            if table_parser.is_none() {
                continue;
            }

            let date = table_parser.as_mut().unwrap().base_date();

            if date.is_none() {
                continue;
            }

            date_map.insert(date.unwrap().clone(), parser);
        }

        return date_map.into_iter()
            .max_by_key(|date_html: &(NaiveDate, &mut html::Parser)| {
                let date = date_html.0;
                date
            })
    }
}
impl Default for Container {
    fn default() -> Self {
        Container::new(vec![], vec![])
    }
}