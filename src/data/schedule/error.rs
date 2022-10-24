/// # Default waffle maker
macro_rules! err {
    ($name: ident, $text: literal) => {
        #[derive(Debug, Clone)]
        pub struct $name;
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, $text)
            }
        }
        impl std::error::Error for $name {}
    };
}

err!(
    ExtractingEmptyContent, 
    "cannot extract empty content"
);

