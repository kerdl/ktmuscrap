use std::path::PathBuf;


#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub string: String
}
