use lazy_static::lazy_static;


lazy_static! {
    static ref NO_SCHEDULES_LOADED: ApiError = {
        ApiError::new(
            Kind::UserFailure, 
            0, 
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
            1, 
            "
            upload GROUP matching regex \
            with POST at /load/regex/group \
            and put regex string in body\
            ".to_owned())
    };
}

pub enum Kind {
    /// ## Indicates user's failure
    /// - i.e. some parameters were not loaded
    UserFailure,
    /// ## Indicates 3rd party failure
    /// - i.e. schedule is formatted incorrectly
    ParsingFailure
}

pub struct ApiError {
    pub kind: Kind,
    pub code: u32,
    pub text: String
}
impl ApiError {
    pub const fn new(kind: Kind, code: u32, text: String) -> ApiError {
        ApiError { kind, code, text }
    }
}

trait ToApiError {
    fn to_api_error(code: u32, text: String) -> ApiError;
}
