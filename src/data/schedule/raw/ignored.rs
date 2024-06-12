use std::{path::PathBuf, collections::HashSet};


pub async fn except(paths: &HashSet<PathBuf>) -> tokio::io::Result<HashSet<PathBuf>> {
    let mut ignored = HashSet::new();

    if paths.is_empty() {
        return Ok(ignored);
    }

    let first_path = paths.iter().nth(0).unwrap();

    let dir = match first_path.clone() {
        p if first_path.is_dir() => p.clone(),
        p if first_path.is_file() => p.parent().unwrap().to_path_buf(),
        _ => unreachable!()
    };

    let mut contents = tokio::fs::read_dir(dir).await?;

    while let Ok(Some(entry)) = contents.next_entry().await {
        if paths.contains(&entry.path()) {
            continue;
        }

        ignored.insert(entry.path());
    }

    Ok(ignored)
}

pub fn except_difference(paths: &HashSet<PathBuf>, except: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    paths.difference(&except).map(|path| path.clone()).collect()
}