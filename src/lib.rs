use std::fmt::Debug;

use itertools::{unfold, Unfold};

pub use errors::StreamParserError;
pub use search_group::{Heuristic, StartGroup};

use crate::logic::{iteration_logic, Logic};
use crate::parser_state::ParserState;
pub use crate::traits::{Buffer, ParserFunction};

pub mod buffers;
mod errors;
mod logic;
mod parser_state;
mod search_group;
mod traits;
mod utils;

/// Define the internal type of the yielded stream
type SteamUnfold<'a, I, B, R> =
    Unfold<ParserState<'a, I, B, R>, Logic<ParserState<'a, I, B, R>, R>>;

/// Public interface of the stream parser
pub struct StreamParser<'a, I, B, R>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
{
    stream: SteamUnfold<'a, I, B, R>,
}

impl<'a, I, B, R> StreamParser<'a, I, B, R>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    R: Debug,
{
    pub fn new(
        iterator: I,
        save_buffer: &'a mut B,
        work_buffer: &'a mut B,
        parser: ParserFunction<R>,
        group_start: Heuristic<'a>,
    ) -> Self {
        let logic_state = ParserState::new(save_buffer, work_buffer, iterator, parser, group_start);

        let stream = unfold(logic_state, iteration_logic());
        StreamParser { stream }
    }
}

impl<'a, I, B, R> Iterator for StreamParser<'a, I, B, R>
where
    I: Iterator<Item = &'a [u8]>,
    B: Buffer,
    R: Debug,
{
    type Item = Result<R, StreamParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next()
    }
}
