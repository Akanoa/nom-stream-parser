use std::ops::{Deref, DerefMut};

use crate::{Buffer, ParserFunction};
use crate::heuristic::Heuristic;
use crate::parser_state::{ParsableState, SearchState};

pub mod async_iterator;
pub mod sync_iterator;
pub mod async_reader;
pub mod sync_reader;

struct ParserCommonFields<'a, B, O, H: Heuristic> {
    /// Parsed buffer
    pub work_buffer: &'a mut B,
    /// Define both whether a new group must be searched
    /// and whether the parser need more data or data are
    /// sufficient to take a decision
    pub state: (SearchState, ParsableState),
    /// Work buffer cursor, define at which position
    /// data in buffer are start to read
    pub cursor: usize,
    /// The master used to generate parsing decision
    /// and result data yielded by stream parser
    pub parser: ParserFunction<O>,
    ///
    pub heuristic: H,
    #[allow(unused)]
    /// Used to debug the system when it comes to infinite loop
    i: usize,
}

impl<'a, B, O, H: Heuristic> ParserCommonFields<'a, B, O, H>
where
    H: Heuristic,
    B: Buffer,
{
    pub fn new(work_buffer: &'a mut B, parser: ParserFunction<O>, heuristic: H) -> Self {
        ParserCommonFields {
            work_buffer,
            state: (SearchState::SearchForStart, ParsableState::NeedMoreData),
            cursor: 0,
            parser,
            heuristic,
            i: 0,
        }
    }
}

impl<'a, B, O, H: Heuristic> Deref for ParserCommonFields<'a, B, O, H>
where
    B: Buffer,
{
    type Target = B;

    fn deref(&self) -> &Self::Target {
        self.work_buffer
    }
}

impl<'a, B, O, H: Heuristic> DerefMut for ParserCommonFields<'a, B, O, H>
where
    B: Buffer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.work_buffer
    }
}
