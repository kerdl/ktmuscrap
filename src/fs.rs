pub mod collect {
    use async_recursion::async_recursion;
    use std::path::PathBuf;

    use crate::SyncResult;

    #[async_recursion]
    pub async fn file_paths(
        dir: &PathBuf
    ) -> tokio::io::Result<Vec<PathBuf>> {
    
        let mut paths = vec![];
    
        let mut entries = tokio::fs::read_dir(dir).await?;
    
        while let Ok(Some(entry)) = entries.next_entry().await {
    
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
    use tokio::task::JoinHandle;
    use std::{path::PathBuf, collections::HashSet};

    use crate::SyncResult;


    pub fn from_set(
        set: HashSet<PathBuf>
    ) -> Vec<JoinHandle<tokio::io::Result<()>>> {
        let mut handles = vec![];

        for path in set {
            let handle = tokio::spawn(async move {
                match path {
                    p if path.is_dir() => tokio::fs::remove_dir_all(p).await?,
                    p if path.is_file() => tokio::fs::remove_file(p).await?,
                    _ => unreachable!()
                }

                Ok::<(), tokio::io::Error>(())
            });

            handles.push(handle);
        }

        handles
    }

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

pub mod hash {
    use sha2::{Sha256, Digest};
    use std::path::PathBuf;

    pub async fn get_sha256(path: &PathBuf) -> tokio::io::Result<String> {
        let html = tokio::fs::read_to_string(path).await?;

        let mut hasher = Sha256::default();
        hasher.update(html.as_bytes());
        let bytes_hash = hasher.finalize();

        let string_hash = hex::encode(bytes_hash);

        Ok(string_hash)
    }
}