pub mod base;

use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use serde_derive::Serialize;
use std::path::PathBuf;

use base::{ApiError, Kind, ToApiError};
use crate::{data::schedule, derive_new::new};


/// # Less boilerplate API error
/// 
/// ## Usage
/// ```
/// api_err!(
///     // struct name
///     name:    InvalidUtf8,
///     // the same but as enum
///     as_enum: ErrorNum::InvalidUtf8,
///     // type of error
///     kind:    Kind::UserFailure,
///     // optional, fields inside struct
///     fields:  (pub error: String, pub sc_type: String),
///     // closure to format error text, using `this` instead of `self`
///     error:   |this| format!(
///         "failed to decode raw bytes to utf-8 with error {:?}", 
///         this.error
///     )
/// );
/// ```
macro_rules! api_err {
        // NotAValidUtf8
    (   name: $name: ident,
        // ErrorNum::NotAValidUtf8
        as_enum: $enum_variant: path,
        // Kind::UserFailure
        kind: $kind: path,
        // (field1: String, pub field2: String)
        // OR NOTHING, FIELDS ARE OPRIONAL
        $(fields: ($($visibility: vis $field: ident: $field_type: ty),*),)?
        // |this| format!("fuck you {}", this.field1)
        error: $error_closure: expr
    ) => {
        #[derive(new, Debug, Clone)]
        pub struct $name {
            $($($visibility $field: $field_type),*)?
        }
        impl ToApiError for $name {
            fn to_api_error(&self) -> ApiError {
                let err = $enum_variant;
                let error_formatter: &dyn Fn(&Self) -> String = &$error_closure;
                let text = error_formatter(self);
        
                ApiError::new($kind, err, text)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let error_formatter: &dyn Fn(&Self) -> String = &$error_closure;
                let text = error_formatter(self);

                write!(f, "{}", text)
            }
        }
        impl std::error::Error for $name {}
    };
}


#[derive(ToPrimitive, Serialize, Clone, Debug)]
pub enum ErrorNum {
    Unknown = 0,
    IoError,

    InvalidUtf8 = 100,

    NoWeeklySchedulesLoaded = 200,
    NoDailySchedulesLoaded,
    ScheduleSavingFailed,
    ScheduleExtractionFailed,
    ScheduleClearFailed,
    RawScheduleDeletionFailed,
    ScheduleDeletionFailed,
    MassScheduleDeletionFailed,
    NoHtmls,
    MultipleHtmls,
    PageParsingFailed,
    NoLastSchedule,
}
impl ErrorNum {
    pub fn to_u32(&self) -> u32 {
        ToPrimitive::to_u32(self).unwrap()
    }
}

api_err!(
    name:    Unknown,
    as_enum: ErrorNum::Unknown,
    kind:    Kind::InternalFailure,
    fields:  (pub text: String),
    error:   |this| format!(
        "wtf dude unknown error: {:?}",
        this.text
    )
);

api_err!(
    name:    IoError,
    as_enum: ErrorNum::IoError,
    kind:    Kind::InternalFailure,
    fields:  (pub text: String),
    error:   |this| format!(
        "io error while converting: {:?}",
        this.text
    )
);

api_err!(
    name:    NoWeeklySchedulesLoaded,
    as_enum: ErrorNum::NoWeeklySchedulesLoaded,
    kind:    Kind::UserFailure,
    error:   |_this| 
        "
        to convert weekly type, \
        load both `ft_weekly` and `r_weekly` \
        raw types: POST ZIP files at \
        /schedule/<type>/load\
        ".to_owned()
);

api_err!(
    name:    NoDailySchedulesLoaded,
    as_enum: ErrorNum::NoDailySchedulesLoaded,
    kind:    Kind::UserFailure,
    error:   |_this| 
        "
        to convert daily type, \
        load both `ft_daily` and `r_weekly` \
        raw types: POST ZIP files at \
        /schedule/<type>/load\
        ".to_owned()
);

api_err!(
    name:    InvalidUtf8,
    as_enum: ErrorNum::InvalidUtf8,
    kind:    Kind::UserFailure,
    fields:  (pub error: String, pub sc_type: String),
    error:   |this| format!(
        "failed to decode raw bytes to utf-8 with error {:?}", 
        this.error
    )
);

api_err!(
    name:    ScheduleExtractionFailed,
    as_enum: ErrorNum::ScheduleExtractionFailed,
    kind:    Kind::UserFailure,
    fields:  (pub sc_type: schedule::raw::Type, pub error: String),
    error:   |this| format!(
        "{} failed to extract with error {:?}", 
        this.sc_type.to_string(), 
        this.error
    )
);

api_err!(
    name:    ScheduleClearFailed,
    as_enum: ErrorNum::ScheduleClearFailed,
    kind:    Kind::InternalFailure,
    fields:  (
        pub sc_type: schedule::raw::Type,
        pub dep_sc_types: Vec<schedule::Type>,
        pub error: String
    ),
    error:   |this| format!(
        "while loading {}, the dependent {:?} failed to clear with error {:?}", 
        this.sc_type.to_string(), 
        this.dep_sc_types,
        this.error
    )
);

api_err!(
    name:    RawScheduleDeletionFailed,
    as_enum: ErrorNum::RawScheduleDeletionFailed,
    kind:    Kind::InternalFailure,
    fields:  (pub sc_type: schedule::raw::Type, pub error: String),
    error:   |this| format!(
        "{} failed to delete from disk with error {:?}", 
        this.sc_type.to_string(), 
        this.error
    )
);

api_err!(
    name:    ScheduleDeletionFailed,
    as_enum: ErrorNum::ScheduleDeletionFailed,
    kind:    Kind::InternalFailure,
    fields:  (pub sc_type: schedule::Type, pub error: String),
    error:   |this| format!(
        "{} failed to delete from disk with error {:?}", 
        this.sc_type.to_string(), 
        this.error
    )
);

api_err!(
    name:    MassScheduleDeletionFailed,
    as_enum: ErrorNum::MassScheduleDeletionFailed,
    kind:    Kind::InternalFailure,
    fields:  (pub error: String),
    error:   |this| format!(
        "failed to mass delete schedule from disk with error {:?}",
        this.error
    )
);

api_err!(
    name:    NoHtmls,
    as_enum: ErrorNum::NoHtmls,
    kind:    Kind::DataFailure,
    fields:  (pub sc_type: schedule::raw::Type),
    error:   |this| format!(
        "{} contains no html files inside archive, wtf dude",
        this.sc_type.to_string()
    )
);

api_err!(
    name:    MultipleHtmls,
    as_enum: ErrorNum::MultipleHtmls,
    kind:    Kind::DataFailure,
    fields:  (pub sc_type: schedule::raw::Type, pub index: Vec<PathBuf>),
    error:   |this| format!(
        "{} contains multiple html files inside archive: {:?}, wtf dude",
        this.sc_type.to_string(),
        this.index
    )
);

api_err!(
    name:    PageParsingFailed,
    as_enum: ErrorNum::PageParsingFailed,
    kind:    Kind::UserFailure,
    fields:  (pub sc_type: schedule::Type, pub error: String),
    error:   |this| format!(
        "the {} page you sent to compare cannot be parsed: {:?}",
        this.sc_type.to_string(),
        this.error
    )
);

api_err!(
    name:    NoLastSchedule,
    as_enum: ErrorNum::NoLastSchedule,
    kind:    Kind::UserFailure,
    fields:  (pub sc_type: schedule::Type),
    error:   |this| format!(
        "the {} cannot be compared since there's nothing to compare with",
        this.sc_type.to_string(),
    )
);