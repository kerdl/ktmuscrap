pub mod base;

use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use serde_derive::Serialize;
use lazy_static::lazy_static;

use base::{ApiError, Kind, ToApiError};
use crate::data::schedule;


#[derive(ToPrimitive, Serialize)]
pub enum ErrorNum {
    NoSchedulesLoaded = 0,
    ScheduleExtractionFailed,

    NoGroupRegexLoaded = 100,
    NoTeacherRegexLoaded,
}
impl ErrorNum {
    pub fn to_u32(&self) -> u32 {
        ToPrimitive::to_u32(self).unwrap()
    }
}


lazy_static! {
    static ref NO_SCHEDULES_LOADED: ApiError = {
        ApiError::new(
            Kind::UserFailure, 
            ErrorNum::NoSchedulesLoaded, 
            "
            upload at least one schedule: \
            `ft_weekly`, `ft_daily`, `r_weekly` \
            with POST at /load/schedule/<schedule type> \
            and put its ZIP content in body\
            ".to_owned()
        )
    };
    static ref NO_GROUP_REGEX_LOADED: ApiError = {
        ApiError::new(
            Kind::UserFailure, 
            ErrorNum::NoGroupRegexLoaded, 
            "
            upload GROUP matching regex \
            with POST at /load/regex/group \
            and put regex string in body\
            ".to_owned())
    };
    static ref NO_TEACHER_REGEX_LOADED: ApiError = {
        ApiError::new(
            Kind::UserFailure, 
            ErrorNum::NoTeacherRegexLoaded, 
            "
            upload TEACHER matching regex \
            with POST at /load/regex/teacher \
            and put regex string in body\
            ".to_owned())
    };
}

pub struct ScheduleExtractionFailed {
    pub sc_type: schedule::raw::Type,
    pub error: String,
}
impl ScheduleExtractionFailed {
    pub fn new(
        sc_type: schedule::raw::Type, 
        error: String
    ) -> ScheduleExtractionFailed{
        ScheduleExtractionFailed { sc_type, error }
    }
}
impl ToApiError for ScheduleExtractionFailed {
    fn to_api_error(&self) -> ApiError {
        let err = ErrorNum::ScheduleExtractionFailed;
        let text = format!(
            "{} failed to extract with error {:?}",
            self.sc_type.to_str(),
            self.error
        );

        ApiError::new(Kind::UserFailure, err, text)
    }
}