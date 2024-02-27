#[macro_export]
macro_rules! debug {
    ($input:expr) => {
        String::from_utf8_lossy($input)
    };
}
