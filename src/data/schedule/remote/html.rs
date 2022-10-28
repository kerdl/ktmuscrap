use chrono::NaiveDate;
use tokio::sync::RwLock;
use std::{path::PathBuf, sync::Arc};

use crate::{parse::remote::html::Html, SyncResult};


pub struct Container {
    pub list: Vec<Html>,
    pub old: Vec<Html>
}
impl Container {
    pub fn new(list: Vec<Html>, old: Vec<Html>) -> Container {
        Container { list, old }
    }

    pub async fn from_paths(paths: Vec<PathBuf>) -> SyncResult<Container> {
        let mut this = Container::default();

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
        self.list.extend_from_slice(&htmls);

        Ok(())
    }

    pub async fn latest(&mut self) -> Option<(NaiveDate, &mut Html)> {
        todo!();
        /* 
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
        */
    }
}
impl Default for Container {
    fn default() -> Self {
        Container::new(vec![], vec![])
    }
}