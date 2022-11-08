pub mod weekly;
pub mod daily;

use actix_web::{web, Responder};
use std::error::Error;

use crate::{
    api::{self, error::base::ToApiError, Response, ToResponse}, 
    data::schedule::{Type, Page},
    LAST,
    compare::{self, DetailedCmp}
};
