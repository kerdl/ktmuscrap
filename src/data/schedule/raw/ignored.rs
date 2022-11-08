use std::{path::PathBuf, collections::HashSet};


pub async fn except(path: &PathBuf) -> tokio::io::Result<HashSet<PathBuf>> {
    let mut ignored = HashSet::new();

    let dir = match path.clone() {
        p if path.is_dir() => p.clone(),
        p if path.is_file() => p.parent().unwrap().to_path_buf(),
        _ => unreachable!()
    };

    let mut contents = tokio::fs::read_dir(dir).await?;

    while let Ok(Some(entry)) = contents.next_entry().await {
        if &entry.path() == path {
            continue;
        }

        ignored.insert(entry.path());
    }

    Ok(ignored)
}