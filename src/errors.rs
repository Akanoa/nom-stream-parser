use nom::error::Error;
use thiserror::Error;

use crate::debug;

#[derive(Debug, PartialEq)]
pub struct ExceedBuffer;

#[derive(Error, Debug)]
pub enum StreamParserError {
    #[error("Parsing error occurred : {0}")]
    Nom(nom::Err<Error<String>>),
    #[error("IO error : {0}")]
    Io(#[from] std::io::Error),
    #[error(
        "Buffer overflow : trying append {data_size} data size into a buffer of size {buffer_size}"
    )]
    ExceededBuffer {
        buffer_size: usize,
        data_size: usize,
    },
    #[error("Buffer overflow : buffer size {buffer_size}")]
    ExceededBufferUnknownSize { buffer_size: usize },
}

impl From<nom::Err<Error<&[u8]>>> for StreamParserError {
    fn from(value: nom::Err<Error<&[u8]>>) -> Self {
        let mapped_error = value.map_input(|error_slice| debug!(error_slice).to_string());
        StreamParserError::Nom(mapped_error)
    }
}
