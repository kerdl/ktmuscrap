pub mod collect {
    use async_recursion::async_recursion;
    use std::path::PathBuf;

    use crate::SyncResult;

    #[async_recursion]
    pub async fn file_paths(
        dir: &PathBuf
    ) -> SyncResult<Vec<PathBuf>> {
    
        let mut paths = vec![];
    
        let mut entries = tokio::fs::read_dir(dir).await?;
    
        while let Some(entry) = entries.next_entry().await? {
    
            if entry.path().is_dir() {
                // collect its files
                let folder_paths = file_paths(&entry.path()).await?;
                // add everything from that dir to our collection
                paths.extend(folder_paths);
            } else {
                paths.push(entry.path())
            }
    
        }
    
        Ok(paths)
    }
    
    pub async fn file_paths_by_extension(
        dir: &PathBuf,
        extension: &str
    ) -> SyncResult<Vec<PathBuf>> {

        let all_files = file_paths(dir).await?;

        let filtered = all_files.iter().filter(
            |path| if let Some(ext) = path.extension() {
                ext == extension
            } else {
                false
            }
        ).cloned();

        Ok(filtered.collect())
    }
}

pub mod remove {
    use log::warn;
    use std::path::PathBuf;

    use crate::SyncResult;


    pub async fn all_except(
        dir: &PathBuf,
        except: &PathBuf
    ) -> SyncResult<()> {
    
        let mut entries = tokio::fs::read_dir(dir).await?;
    
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
    
            if &path == except {
                continue;
            }
    
            tokio::spawn(async move {
                if path.is_dir() {
                    if let Err(err) = tokio::fs::remove_dir_all(&path).await {
                        warn!("error removing directory {:?}: {:?}", path, err);
                    }
                } else if path.is_file() {
                    if let Err(err) = tokio::fs::remove_file(&path).await {
                        warn!("error removing file {:?}: {:?}", path, err);
                    }
                }
            });
        }
    
        Ok(())
    }
}
