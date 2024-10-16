pub mod base;

use num_derive::ToPrimitive;
use num_traits::ToPrimitive;
use serde_derive::Serialize;

use base::{ApiError, Kind, ToApiError};
use crate::{data::schedule, derive_new::new};


/// # Less boilerplate API error
/// 
/// ## Usage
/// ```
/// api_err!(
///     // struct name
///     name:    YouWereAnError,
///     // the same but as enum
///     as_enum: ErrorNum::YouWereAnError,
///     // type of error
///     kind:    Kind::UserFailure,
///     // optional, fields inside struct
///     fields:  (pub error: String, pub other: String),
///     // closure to format error text, using `this` instead of `self`
///     error:   |this| format!(
///         "i wish you were never born {:?}", 
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
        // OR NOTHING, FIELDS ARE OPTIONAL
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
    NoLastSchedule = 100
}
impl ErrorNum {
    pub fn to_u32(&self) -> u32 {
        ToPrimitive::to_u32(self).unwrap()
    }
}

api_err!(
    name:    NoLastSchedule,
    as_enum: ErrorNum::NoLastSchedule,
    kind:    Kind::InternalFailure,
    fields:  (pub kind: schedule::raw::Kind),
    error:   |_this| format!(
        "no schedule found, make sure tables are still available and are valid"
    )
);
