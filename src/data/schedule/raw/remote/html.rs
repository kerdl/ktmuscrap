use derive_new::new;
use chrono::NaiveDate;
use tokio::sync::RwLock;
use std::{path::PathBuf, sync::Arc, collections::HashMap};

use crate::{parse::remote::html, SyncResult};


#[derive(new)]
pub struct Container {
    pub list: Vec<html::Parser>
}
impl Container {
    pub async fn from_paths(paths: Vec<PathBuf>) -> SyncResult<Container> {
        let mut this = Container::default();

        this.add_from_paths(paths).await?;

        Ok(this)
    }

    pub async fn add_from_paths(
        &mut self, 
        paths: Vec<PathBuf>
    ) -> SyncResult<()> {

        let mut htmls = vec![];
        let mut handles = vec![];

        for path in paths {

            let handle = tokio::spawn(async move {
                let path = path.clone();

                let html = html::Parser::from_path(path).await?;

                Ok::<
                    html::Parser,
                    Box<dyn std::error::Error + Send + Sync>
                >(html)
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        for handle in handles {
            let html = handle.await??;
            htmls.push(html);
        }

        // add everything to `self` container
        self.list.extend_from_slice(&htmls);

        Ok(())
    }

    pub async fn date_path_map(&mut self) -> HashMap<PathBuf, NaiveDate> {
        let mut date_path = HashMap::new();

        for parser in self.list.iter_mut() {

            let path = parser.path.clone();

            let table = parser.table();
            if table.is_none() { continue; }

            let base_date = table.unwrap().base_date();
            if base_date.is_none() { continue; }

            date_path.insert(
                path,
                base_date.unwrap().clone(),
            );
        }

        date_path
    }

    pub async fn latest_path(&mut self) -> Option<(PathBuf, NaiveDate)> {

        self.date_path_map().await.into_iter()
            .max_by_key(|path_date: &(PathBuf, NaiveDate)| {
                let date = path_date.1;
                date
            })
    }

    pub async fn get_by_path(
        &mut self, 
        path: &PathBuf
    ) -> Option<&mut html::Parser> {

        self.list.iter_mut().find(|parser| {
            &parser.path == path
        })
    }

    pub async fn latest(&mut self) -> Option<(NaiveDate, &mut html::Parser)> {
        let latest = self.latest_path().await?;

        let latest_path = latest.0;
        let latest_date = latest.1;
    
        let parser = self.get_by_path(&latest_path).await?;

        Some((latest_date, parser))
    }

    pub async fn clear_old(&mut self) -> Option<Vec<PathBuf>> {
        let mut removed_paths = vec![];

        let latest_path = self.latest_path().await?.0;

        while let Some(old_index) = {
            self.list.iter()
            .position(|parser| parser.path != latest_path)
        } {
            let item = self.list.get_mut(old_index)?;
            let path = item.path.clone();

            removed_paths.push(path);

            self.list.remove(old_index);
        }

        Some(removed_paths)
    }
}
impl Default for Container {
    fn default() -> Self {
        Container::new(vec![])
    }
}