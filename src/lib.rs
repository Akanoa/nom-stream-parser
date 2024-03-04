#![doc = include_str!("README.md")]

use std::fmt::Debug;
use std::io::Read;

use itertools::{unfold, Unfold};

pub use errors::StreamParserError;
pub use heuristic::{Heuristic, StartGroup};

use crate::logic::{iteration_logic, Logic};
use crate::parser_state::ParserState;
pub use crate::traits::{Buffer, ParserFunction, ParserFunctionStartGroup};

pub mod buffers;
#[cfg(feature = "builder")]
pub mod builder;
mod errors;
mod heuristic;
mod logic;
mod parser_state;
pub mod stream_parsers;
mod traits;
mod utils;

pub enum DataSource<'a, I, R>
where
    I: Iterator<Item = &'a [u8]>,
    R: Read,
{
    Iterator(I),
    Reader(R),
}

/// Define the internal type of the yielded stream
type SteamUnfold<'a, I, B, R, O> =
    Unfold<ParserState<'a, I, B, R, O>, Logic<ParserState<'a, I, B, R, O>, O>>;

/// Public interface of the stream parser
pub struct StreamParser<'a, I, B, R, O>
where
    I: Iterator<Item = &'a [u8]>,
    R: Read,
    B: Buffer,
    O: Debug,
{
    stream: SteamUnfold<'a, I, B, R, O>,
}

impl<'a, I, B, R, O> StreamParser<'a, I, B, R, O>
where
    I: Iterator<Item = &'a [u8]>,
    R: Read,
    B: Buffer,
    O: Debug,
{
    pub fn new(
        data_source: DataSource<'a, I, R>,
        work_buffer: &'a mut B,
        parser: ParserFunction<O>,
        group_start: Heuristic<'a>,
    ) -> Self {
        let logic_state = ParserState::new(work_buffer, data_source, parser, group_start);

        let stream = unfold(logic_state, iteration_logic());
        StreamParser { stream }
    }
}

impl<'a, I, B, R, O> Iterator for StreamParser<'a, I, B, R, O>
where
    I: Iterator<Item = &'a [u8]>,
    R: Read,
    B: Buffer,
    O: Debug,
{
    type Item = Result<O, StreamParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next()
    }
}
