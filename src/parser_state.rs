use std::cell::RefCell;
use std::io::Read;

use crate::heuristic::Heuristic;
use crate::traits::{Buffer, ParserFunction};
use crate::DataSource;

/// Define the decision of the master parser at previous iteration
#[derive(Debug)]
pub(crate) enum ParsableState {
    /// The data in the work aren't enough to decide the parsing state
    NeedMoreData,
    /// The data in working buffer may lead to parsing decision
    MaybeParsable,
}

/// Command whether the search start group must be run
#[derive(Debug)]
pub(crate) enum SearchState {
    /// We are still searching for relevant data to parse
    SearchForStart,
    /// The start of a relevant data to parse have found
    StartFound,
}

pub(crate) struct ParserState<'a, I, B, R, O>
where
    I: Iterator<Item = &'a [u8]>,
    R: Read,
    B: Buffer,
{
    /// Iterated data
    pub data_source: DataSource<'a, I, R>,
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
    pub heuristic: RefCell<Heuristic<'a>>,
    #[allow(unused)]
    /// Used to debug the system when it comes to infinite loop
    i: usize,
}

impl<'a, I, B, R, O> ParserState<'a, I, B, R, O>
where
    I: Iterator<Item = &'a [u8]> + Iterator<Item = &'a [u8]>,
    B: Buffer,
    R: Read,
{
    pub(crate) fn new(
        work_buffer: &'a mut B,
        data_source: DataSource<'a, I, R>,
        parser: ParserFunction<O>,
        start_group: Heuristic<'a>,
    ) -> Self {
        Self {
            data_source,
            work_buffer,
            state: (SearchState::SearchForStart, ParsableState::NeedMoreData),
            cursor: 0,
            parser,
            heuristic: RefCell::new(start_group),
            i: 0,
        }
    }
}
