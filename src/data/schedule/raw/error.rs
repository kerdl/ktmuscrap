
#[derive(Debug)]
pub enum UnpackError {
    Io(tokio::io::Error),
    Zip(async_zip::error::ZipError)
}