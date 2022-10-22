pub mod load;
pub mod error;

use error::ApiError;


pub struct Data {

}

pub struct Response {
    is_ok: bool,
    data: Data,
    error: ApiError,
}
impl Response {
    pub fn new(is_ok: bool, data: Data, error: ApiError) -> Response {
        Response { is_ok, data, error }
    }
}