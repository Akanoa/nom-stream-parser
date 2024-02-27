use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use crate::search_group::Heuristic;
use crate::traits::{Buffer, ParserFunction};

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

pub(crate) struct ParserState<'a, I, B, R>
    where
        I: Iterator<Item = &'a [u8]>,
        B: Buffer,
{
    /// Iterated data
    pub iterator: I,
    /// Buffer used when data must be accumulated
    pub save_buffer: &'a mut B,
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
    pub parser: ParserFunction<R>,
    ///
    pub heuristic: RefCell<Heuristic<'a>>,
    #[allow(unused)]
    /// Used to debug the system when it comes to infinite loop
    i: usize,
}

impl<'a, I, B, R> Deref for ParserState<'a, I, B, R>
    where
        B: Buffer,
        I: Iterator<Item = &'a [u8]>,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.iterator
    }
}

impl<'a, I, B, R> DerefMut for ParserState<'a, I, B, R>
    where
        I: Iterator<Item = &'a [u8]>,
        B: Buffer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iterator
    }
}

impl<'a, I, B, R> ParserState<'a, I, B, R>
    where
        I: Iterator<Item = &'a [u8]> + Iterator<Item = &'a [u8]>,
        B: Buffer,
{
    pub(crate) fn new(
        save_buffer: &'a mut B,
        work_buffer: &'a mut B,
        iterator: I,
        parser: ParserFunction<R>,
        start_group: Heuristic<'a>,
    ) -> Self {
        Self {
            iterator,
            save_buffer,
            work_buffer,
            state: (
                SearchState::SearchForStart,
                ParsableState::NeedMoreData,
            ),
            cursor: 0,
            parser,
            heuristic: RefCell::new(start_group),
            i: 0,
        }
    }
}