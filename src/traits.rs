use std::ops::Deref;
use crate::errors::StreamParserError;

/// Define a parser function used to generate data of the final stream
pub type ParserFunction<R> = fn(&[u8]) -> nom::IResult<&[u8], R>;
/// Define a parser which found the start of a group of data
pub type ParserFunctionStartGroup = fn(&[u8]) -> nom::IResult<&[u8], &[u8]>;

/// Define the behavior expected by a buffer used while parsing data
pub trait Buffer: Deref<Target = [u8]> {
    /// Add data to buffer, if evincealble declares an amount of data removable
    fn append(&mut self, other: &[u8], evinceable: Option<usize>) -> Result<bool, StreamParserError>;
    /// Copy the data from another buffer
    fn copy_from(&mut self, source: &Self, evinceable: Option<usize>);
    /// Clean data of the buffer
    fn clear(&mut self);
}