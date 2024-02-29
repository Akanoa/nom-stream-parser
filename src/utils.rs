/// Allow to debug slice of bytes as String representation
/// ```
/// use nom_stream_parser::debug;
/// let data = b"abc125)";
/// let result = debug!(data);
/// assert_eq!("abc125)".to_string(), result);
/// ```
#[macro_export]
macro_rules! debug {
    ($input:expr) => {
        String::from_utf8_lossy($input)
    };
}
