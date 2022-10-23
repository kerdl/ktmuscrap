pub mod base;

use std::collections::HashSet;

use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use serde_derive::Serialize;
use lazy_static::lazy_static;

use base::{ApiError, Kind, ToApiError};
use crate::data::{schedule, regex};


#[derive(ToPrimitive, Serialize, Clone, Debug)]
pub enum ErrorNum {
    NoSchedulesLoaded = 0,
    ScheduleExtractionFailed,
    ScheduleDeletionFailed,
    MassScheduleDeletionFailed,

    RegexNotAValidUtf8 = 100,
    RegexCompileFailed,
    RegexesNotSet,
}
impl ErrorNum {
    pub fn to_u32(&self) -> u32 {
        ToPrimitive::to_u32(self).unwrap()
    }
}


lazy_static! {
    pub static ref NO_SCHEDULES_LOADED: ApiError = {
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

pub struct ScheduleDeletionFailed {
    pub sc_type: schedule::raw::Type,
    pub error: String,
}
impl ScheduleDeletionFailed {
    pub fn new(
        sc_type: schedule::raw::Type, 
        error: String
    ) -> ScheduleDeletionFailed{
        ScheduleDeletionFailed { sc_type, error }
    }
}
impl ToApiError for ScheduleDeletionFailed {
    fn to_api_error(&self) -> ApiError {
        let err = ErrorNum::ScheduleDeletionFailed;
        let text = format!(
            "{} failed to delete from disk with error {:?}",
            self.sc_type.to_str(),
            self.error
        );

        ApiError::new(Kind::UserFailure, err, text)
    }
}

pub struct MassScheduleDeletionFailed {
    pub error: String,
}
impl MassScheduleDeletionFailed {
    pub fn new(error: String) -> MassScheduleDeletionFailed{
        MassScheduleDeletionFailed { error }
    }
}
impl ToApiError for MassScheduleDeletionFailed {
    fn to_api_error(&self) -> ApiError {
        let err = ErrorNum::MassScheduleDeletionFailed;
        let text = format!(
            "failed to mass delete schedule from disk with error {:?}",
            self.error
        );

        ApiError::new(Kind::UserFailure, err, text)
    }
}

pub struct RegexNotAValidUtf8 {
    pub regex_type: regex::Type,
    pub error: String
}
impl RegexNotAValidUtf8 {
    pub fn new(
        regex_type: regex::Type,
        error: String
    ) -> RegexNotAValidUtf8 {
        RegexNotAValidUtf8 { regex_type, error }
    }
}
impl ToApiError for RegexNotAValidUtf8 {
    fn to_api_error(&self) -> ApiError {
        let err = ErrorNum::RegexNotAValidUtf8;
        let text = format!(
            "failed to decode raw bytes to utf-8 of {} regex with error {:?}",
            self.regex_type.to_str(),
            self.error
        );

        ApiError::new(Kind::UserFailure, err, text)
    }
}

pub struct RegexCompileFailed {
    pub regex_type: regex::Type,
    pub error: String
}
impl RegexCompileFailed {
    pub fn new(
        regex_type: regex::Type,
        error: String
    ) -> RegexCompileFailed {
        RegexCompileFailed { regex_type, error }
    }
}
impl ToApiError for RegexCompileFailed {
    fn to_api_error(&self) -> ApiError {
        let err = ErrorNum::RegexCompileFailed;
        let text = format!(
            "{} regex failed to compile with error {:?}",
            self.regex_type.to_str(),
            self.error
        );

        ApiError::new(Kind::UserFailure, err, text)
    }
}

pub struct RegexesNotSet {
    pub types: HashSet<regex::Type>
}
impl RegexesNotSet {
    pub fn new(types: HashSet<regex::Type>) -> RegexesNotSet {
        RegexesNotSet { types }
    }
}
impl ToApiError for RegexesNotSet {
    fn to_api_error(&self) -> ApiError {
        let err = ErrorNum::RegexesNotSet;
        let text = format!(
            "{:?} regexes are not set, they're all necessary",
            self.types
        );

        ApiError::new(Kind::UserFailure, err, text)
    }
}