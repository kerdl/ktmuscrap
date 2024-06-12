use derive_new::new;
use chrono::NaiveDate;
use tokio::sync::RwLock;
use std::{path::PathBuf, sync::Arc, collections::{HashMap, HashSet}};

use crate::{parse::remote::html, SyncResult, schedule::{raw, update}};


#[derive(new)]
pub struct Container {
    pub list: Vec<html::Parser>,
    pub tchr_list: Vec<html::TchrParser>
}
impl Container {
    pub async fn from_paths(
        paths: Vec<PathBuf>,
        tchr_paths: Vec<PathBuf>
    ) -> SyncResult<Container> {
        let mut this = Container::default();

        let _ = this.add_from_paths(paths).await;
        let _ = this.add_from_tchr_paths(tchr_paths).await;

        Ok(this)
    }

    pub async fn from_files(
        files: Vec<update::File>,
        tchr_files: Vec<update::File>
    ) -> SyncResult<Container> {
        let mut this = Container::default();

        let _ = this.add_from_files(files).await;
        let _ = this.add_from_tchr_files(tchr_files).await;

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

    pub async fn add_from_files(
        &mut self, 
        files: Vec<update::File>
    ) -> SyncResult<()> {
        let mut htmls = vec![];
        let mut handles = vec![];

        for file in files {
            let handle = tokio::spawn(async move {
                let html = html::Parser::from_file(&file).await?;
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

    pub async fn add_from_tchr_paths(
        &mut self, 
        paths: Vec<PathBuf>
    ) -> SyncResult<()> {
        let mut htmls = vec![];
        let mut handles = vec![];

        for path in paths {
            let handle = tokio::spawn(async move {
                let path = path.clone();

                let html = html::TchrParser::from_paths(&[path]).await?;

                Ok::<
                    Vec<html::TchrParser>,
                    Box<dyn std::error::Error + Send + Sync>
                >(html)
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        for handle in handles {
            let mut html = handle.await??;
            htmls.append(&mut html);
        }

        // add everything to `self` container
        self.tchr_list.extend_from_slice(&htmls);

        Ok(())
    }

    pub async fn add_from_tchr_files(
        &mut self, 
        files: Vec<update::File>
    ) -> SyncResult<()> {
        let mut htmls = vec![];
        let mut handles = vec![];

        for file in files {
            let handle = tokio::spawn(async move {
                let v = vec![file];
                let html = html::TchrParser::from_files(&v).await?;
                Ok::<
                    Vec<html::TchrParser>,
                    Box<dyn std::error::Error + Send + Sync>
                >(html)
            });

            handles.push(handle);
        }

        // wait for all tasks to finish
        for handle in handles {
            let mut html = handle.await??;
            htmls.append(&mut html);
        }

        // add everything to `self` container
        self.tchr_list.extend_from_slice(&htmls);

        Ok(())
    }

    pub async fn date_path_map(&mut self, mode: raw::Mode) -> HashMap<PathBuf, NaiveDate> {
        let mut date_path = HashMap::new();

        match mode {
            raw::Mode::Groups => {
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
            },
            raw::Mode::Teachers => {
                for parser in self.tchr_list.iter_mut() {
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
            },
        }

        date_path
    }

    pub async fn latest_paths(&mut self, mode: raw::Mode) -> HashSet<(PathBuf, NaiveDate)> {
        let mut maxes = HashSet::new();
        let map = self.date_path_map(mode).await;

        let Some(max) = map.iter().max_by_key(
            |path_date| path_date.1
        ) else {
            return maxes;
        };

        let other_maxes = map.iter()
            .filter(|path_date| path_date.1 == max.1)
            .map(|path_date| (path_date.0.clone(), path_date.1.clone()))
            .collect::<HashSet<(PathBuf, NaiveDate)>>();

        maxes.insert((max.0.clone(), max.1.clone()));
        maxes.extend(other_maxes);

        maxes
    }
}
impl Default for Container {
    fn default() -> Self {
        Container::new(vec![], vec![])
    }
}