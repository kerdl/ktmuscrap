
#[derive(Debug)]
pub enum UnpackError {
    Io(tokio::io::Error),
    Zip(zip::result::ZipError)
}